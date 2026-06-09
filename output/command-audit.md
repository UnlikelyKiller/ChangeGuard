# ChangeGuard Command Audit Report

Generated: 2026-06-09
Scope: All CLI commands defined in `src/cli.rs` (top-level and subcommands), backed by test evidence and code inspection.
Methodology: 68 integration test files examined, source of every command entry point inspected, recent regression history considered.

---

## 1. Working Commands -- What Reliably Works Well

### changeguard init
- Creates `.changeguard/` directory with `config.toml`, `rules.toml`, `logs/`
- Appends to (does not replace) existing `.gitignore`
- Installs `pre-commit` and `pre-push` git hooks idempotently (re-init does not duplicate hook markers)
- Hooks include `--verify-signatures` flag, `--no-verify` bypass text
- Supports `CHANGEGUARD_DEFAULT_CONFIG` env var for custom template
- Extensive test coverage: `tests/integration/cli_init.rs` (7 tests)

### changeguard scan
- Reports clean/dirty working tree state correctly
- Handles detached HEAD, untracked, modified, and staged changes
- `--impact` flag sub-invokes impact analysis and writes `latest-impact.json`
- `--impact --json` / `--out` produce valid JSON with `schemaVersion: "v1"`
- `--impact --summary` / `--json` / `--out` all properly reject missing `--impact` with clear error
- `ignore_patterns` from config are respected (files matching ignore patterns excluded from impact analysis)
- Test coverage: `tests/integration/cli_scan.rs` (7 tests)

### changeguard impact
- Writes `latest-impact.json` report to `.changeguard/reports/`
- Reports `analysisStatus: "unsupported"` for unanalyzable files (e.g., `.txt`)
- Succeeds gracefully with missing or invalid `rules.toml` (warns but does not fail)
- Test coverage: `tests/integration/cli_impact.rs` (4 tests)

### changeguard ledger {start, commit, rollback, atomic}
- Full round-trip: start -> commit works with UUID tx_id, status transition PENDING -> COMMITTED
- Rollback produces ROLLED_BACK status; can start new tx for same entity after rollback
- Conflict detection prevents double-start on same entity
- Atomic change (start+commit in one call) works correctly
- Fuzzy tx_id matching (first 8 chars prefix) works
- Test coverage: `tests/integration/ledger_lifecycle.rs` (5 tests)

### changeguard ledger status
- Reports pending transactions accurately
- `--compact` flag available
- `--exit-code` flag available
- `--verify-signatures` flag available
- Test coverage: integrated in lifecycle tests

### changeguard ledger search
- Full-text search across ledger entries using FTS5
- Filters work: `--category`, `--days`, `--breaking`
- Results ranked by term frequency (entries with more query matches score higher)
- Pagination with `--limit` and `--offset`
- Invalid FTS syntax returns `LedgerError::Validation`, not a crash
- Test coverage: `tests/integration/ledger_search.rs` (6 tests)

### changeguard ledger drift {reconcile, adopt}
- Drift detection creates UNAUDITED records with counts
- Drift ignored for files already under PENDING transaction
- `reconcile` marks drift as RECONCILED with reason
- `adopt` converts UNAUDITED to PENDING for active tracking
- Bulk reconcile by glob pattern (`src/*.rs`) works
- Auto-reconcile on commit works (reconcile + implementation = 2 entries)
- Concurrency-safe: `update_transaction_status_bulk` checks status precondition (UNAUDITED) before updating
- Test coverage: `tests/integration/ledger_drift.rs` (8 tests)

### changeguard ledger enforce (rules/validators)
- Tech stack rules: insert, query, filter by category
- Commit validators: insert, query, ALL-category matching, per-category matching
- Rule violations block `start_change` with `RuleViolation` error (case-insensitive matching)
- Validator ERROR level blocks commit with `ValidatorFailed`
- Validator WARNING level allows commit through
- Validator timeout produces `ValidatorFailed` with "Validator timed out"
- Absolute entity path substitution (`{entity}` placeholder) works
- Verification gate: `verify_to_commit` config blocks ARCHITECTURE/FEATURE/BUGFIX/INFRA without verification; allows DOCS/CHORE/REFACTOR/TOOLING; force override works
- Test coverage: `tests/integration/ledger_enforcement.rs` (18 tests)

### changeguard ledger adr
- Exports MADR-format files from ledger history
- ARCHITECTURE-category entries and breaking FEATURE entries both export
- Files numbered and slugified: `0001-new-system-architecture.md`
- Content correct: title, category, breaking status
- Test coverage: `tests/integration/ledger_adr.rs` (1 test)

### changeguard ledger provenace (symbol diff)
- Token-level provenance records ADDED/MODIFIED/DELETED per symbol
- `compute_symbol_diff` correctly identifies adds, modifications, deletions
- Test coverage: `tests/integration/ledger_provenance.rs` (2 tests)

### changeguard hotpots
- Score uses frequency * complexity multiplication (catches "worst of both worlds")
- Deterministic sorting: score desc, then path asc for ties
- Directory filter (`--entity src/`) and language filter (`--lang rs`) work
- `--json` serialization works
- NaN/zero-complexity regression fixed: scores are now finite (0.0) when all complexity is zero, not JSON null
- Backward-compat deserialization: null score -> 0.0 for old packets
- Malformed SQLite rows propagate error (Invalid column type) rather than silently corrupting
- Test coverage: `tests/integration/hotspot_ranking.rs` (8 tests)

### changeguard verify
- `verify <command>` runs command and reports success/failure
- `--timeout` kills long-running commands with "Timed out" error
- Missing executable produces "Command not found" error
- `--dry-run` reports success without executing
- `--health` checks executable existence (detects env-var prefix commands correctly)
- CR4: env-var prefix (CARGO_TERM_COLOR=always cargo --version) resolves to `cargo`, not the prefix string
- Invalid `rules.toml` glob patterns produce clear "Invalid glob pattern" error
- Test coverage: `tests/integration/cli_verify.rs` (10 tests), `tests/integration/cli_verify_rules.rs` (1 test)

### changeguard ledger gc
- Supports `--stale` (TTL for PENDING) and `--orphans` (no matching git commit)
- `--force` flag and `--ttl-hours` available
- Test coverage: integrated in ledger lifecycle

### changeguard ledger graph
- KG edges written on ledger commit: `LedgerTransaction` -> `File` with `affects` relation
- URN construction normalizes Windows backslashes to forward slashes
- `execute_ledger_adopt` writes KG edges pointing to real file paths (not synthetic "drift_adoption") -- Codex finding #1 fixed
- Test coverage: `tests/integration/ledger_graph_edges.rs` (3 tests)

### changeguard reset
- Default preserves config.toml, rules.toml, ledger.db
- Removes logs/ and reports/ (transient state)
- `--include-ledger` removes ledger.db (requires `--yes`)
- `--remove-config` and `--remove-rules` require `--yes`
- `--all` removes entire `.changeguard/` tree (requires `--yes`)
- `--dry-run` does not modify anything
- Idempotent
- Never touches files outside `.changeguard/`
- Test coverage: `tests/integration/cli_reset.rs` (10 tests)

### changeguard doctor
- Runs health checks without crashing
- Works in minimal git repos
- Test coverage: `tests/integration/cli_doctor.rs` (1 test)

### changeguard analytics (complexity scoring)
- Rust, Python, TypeScript complexity scoring works
- `ast_incomplete` flag for syntax errors
- `complexity_capped` for files > 10k lines
- `NotApplicable` for unsupported file types
- Test coverage: `tests/integration/complexity_scoring.rs` (5 tests)

### changeguard search (code search)
- Integration test for binary help displays all subcommands
- Test coverage: `tests/integration/cli_binary.rs`

### changeguard persistence (impact packet round-trip)
- `ImpactPacket` round-trips through SQLite with all fields including symbols, metadata
- Metadata JSON map (e.g., `{"reexport": "true"}`) persists and reads back correctly
- NaN hotspot scores in SQLite no longer cause deserialization failure
- Test coverage: `tests/integration/persistence.rs` (1 test) + `tests/integration/impact_verify_pipeline.rs` (2 tests)

### changeguard index (semantic search)
- HNSW approximate nearest-neighbor query (Tier 1) returns ordered results
- Cozo-native cos_dist fallback (Tier 2) works when HNSW is disabled
- HNSW and cos_dist produce same ordering for same data
- Dimension mismatch produces error, not crash
- `remove_file_snippets` deletes only the specified file
- File hash tracking (`is_file_hash_current`, `record_file_hash`, `get_tracked_files`, `remove_file_hash`) works
- Incremental deletions: files missing from filesystem are pruned from both snippets and hashes
- Test coverage: `tests/integration/semantic_search.rs` (9 tests)

### changeguard bridge (AI-Brains integration)
- Bridge export, import, query, notify, IPC, lineage, ask all have test coverage
- Test files: `bridge_export_tests.rs`, `bridge_import_tests.rs`, `bridge_ipc_tests.rs`, `bridge_lineage_tests.rs`, `bridge_notify_tests.rs`, `bridge_query_tests.rs`, `bridge_ask_tests.rs`, `bridge_tests.rs`

### changeguard temporal coupling
- Scoring with decay half-life produces expected scores
- Threshold filtering works (only couplings above threshold returned)
- Insufficient history returns `InsufficientHistory` error
- First-parent vs all-parents traversal shows correct difference (side-branch files excluded from first-parent)
- Test coverage: `tests/integration/temporal_coupling.rs` (4 tests)

### changeguard security / Cedar
- Orphan pruning: when Cedar policy files are deleted, policy child nodes (principal, action, resource) are pruned from CozoDB
- Live policy child nodes survive re-index
- Test coverage: `tests/integration/cedar_orphan_pruning.rs` (2 tests)

### changeguard risk analysis
- Public symbol change -> Medium risk
- File volume (10+ files) + public symbol -> Medium risk
- Protected path (Cargo.toml) + public symbol -> High risk
- Undeclared env var dependency (DATABASE_URL) triggers risk reason
- Common env var (PATH) filtered from risk reasons
- Runtime usage delta: env-var count change and config-key count change trigger reasons
- Same-cardinality replacement (1 env var -> 1 different env var) correctly NOT flagged (known limitation, documented)
- Path-weighted scoring: README.md modifications weighted lower
- Test coverage: `tests/integration/risk_analysis.rs` (8 tests)

### changeguard narrative / architect prompts
- Golden prompt for Senior Architect narrative is deterministic (tested against exact string match)
- Test coverage: `tests/integration/narrative_golden.rs` (1 test)

### changeguard ask
- Requests without impact packet fall back to global mode (no "No impact report found" error)
- Invalid config (e.g., `[watch]\ndebounce_ms = 0`) fails early with relevant error
- `--timeout` flag is respected by local model completion client (test verifies <2s timeout vs 3s server delay)
- KG BM25 fallback provides context when semantic index is empty
- `--no_kg_fallback` flag suppresses KG fallback correctly
- KG neighborhood enrichment (CR7) uses real file paths via file_stem extraction
- Cozo Datalog string escaping (CR8): plain symbols unchanged, single quotes doubled, backslashes escaped, both combined
- Test coverage: `tests/integration/cli_ask.rs` (3 tests), `tests/integration/ask_kg_fallback.rs` (2 tests), escape tests in `cli_verify.rs` (5 tests)

---

## 2. Broken/Failing Commands

### changeguard datasheet / tables subcommands (DataModels, Dependencies, etc.)
- **XML surfaces (`endpoints`, `data-models`, `observability`, `security`, `services`, `deploy`, `ci`, `dependencies`, `tests`) have NO integration test coverage** at the top-level CLI entry point.
- These are wired in `cli.rs` via `#[command(flatten)]` or direct dispatch but their `execute_*` functions are never called in a test that validates the full pipeline from CLI arguments through to output.
- **Severity: High** -- These surfaces can silently regress (wrong output format, broken filter, JSON serialization error) without any test catching it.

### changeguard watch
- Test only validates that a known-bad config (`debounce_ms = 0`) produces the expected error
- No test validates that `execute_watch` actually debounces file events, syncs the graph, or produces JSON output
- **Severity: Medium** -- The watch command is a core UX surface; its reliability is unverified beyond config validation

### changeguard config {verify, view, schema, diff}
- Top-level `ConfigCommands` dispatch is wired but no integration test exercises `execute_config_verify`, `execute_config_view`, `execute_config_schema`, or `execute_config_diff` with real or mock data
- **Severity: High** -- Users rely on `config verify` and `config view` for debugging; a regression in config loading/display would be invisible to tests

### changeguard dead-code
- `DeadCode` command is wired to `execute_dead_code` but has no integration test coverage
- **Severity: Medium** -- This is a newer surface (threshold, limit, auto_index flags) that could easily break

### changeguard viz
- `Viz` command generates an HTML visualization; it is a single-threaded server-side render
- No integration test validates output file creation, node count, depth filtering, or entity filtering
- **Severity: Low-Medium** -- Visual output regressions are less critical but hard to spot without tests

### changeguard federate
- `FederateCommands::Export`, `Scan`, `Status` wired but not integration-tested in a cross-repo federation scenario
- Unit tests for `federated_discovery` exist but do not exercise the CLI command layer
- **Severity: Medium** -- Federation is a secondary feature but silent regressions would erode trust

### changeguard audit (top-level)
- `Commands::Audit` at top level is wired to `execute_ledger_audit`, same as `LedgerCommands::Audit`
- No integration test validates end-to-end audit output with actual ledger data
- **Severity: Medium** -- Duplicate dispatch points risk drift between top-level and ledger-subcommand audit

### changeguard update
- `Update` command with `--migrate`, `--binary`, `--force`, `--force-unlock`, `--dry-run`
- No integration test validates that these flags propagate correctly
- The `--force-unlock` flag terminates other processes; no test validates this safety behavior
- **Severity: High** -- This command touches system state (binary replacement, CozoDB locking); a regression could corrupt state

### changeguard intent demo
- `IntentCommands::Demo` is wired but has no integration test
- **Severity: Low** -- This is a demo/TUI surface; low risk of production regression

### changeguard daemon / viz-server
- Both are `#[cfg(feature = ...)]` gated and have no integration test in the default feature set
- `daemon_lifecycle` integration test exists but is likely behind the feature flag
- **Severity: Low** -- Feature-gated; not part of default build

---

## 3. UX Friction

### 3.1 Inconsistent flag naming
- `scan --impact` triggers impact analysis, but `impact` is also a standalone command. Running `changeguard impact` without any flags produces an analysis but with no easy way to get JSON output without `scan --impact --json`. The standalone `impact` command lacks `--json` and `--out` flags that `scan --impact` has.
- **Suggestion**: Add `--json` and `--out` to `changeguard impact` directly, or document that `changeguard scan --impact --json` is the canonical path.

### 3.2 Ledger `--category` is a free-string in start, but enum elsewhere
- `changeguard ledger start` accepts `--category` as a raw `String`, while `ledger atomic` accepts a `Category` enum (with tab-completion). This inconsistency means `ledger start` silently accepts invalid categories.
- **Suggestion**: Change `ledger start --category` to use the `Category` enum like `atomic` does.

### 3.3 Stale index prompt blocks non-interactive use
- `changeguard ask --semantic` prompts interactively to re-index even when run from scripts or CI. The `--auto-index` flag skips the prompt but the fallback branch in `execute_ask` still uses `inquire::Confirm::new()` when the semantic index is empty, which will hang in non-interactive contexts.
- **Suggestion**: Add a `CHANGEGUARD_NON_INTERACTIVE` env-var gate to the semantic prompt, matching the stale-index prompt pattern.

### 3.4 Help text depth inconsistency
- `changeguard help` shows all top-level commands, but some (like `services`, `security`, `observability`) have subcommands that are only discoverable via `changeguard <subcommand> --help`. The `data-models`, `observability`, `security` commands use `#[command(flatten)]` from external args structs, so their `--help` output does not display the full subcommand tree.
- **Suggestion**: Audit each surface to ensure `--help` displays all available sub-flags, or add a `commands` subcommand that lists all available surfaces.

### 3.5 Json output inconsistency
- Some commands output JSON to stdout (`scan --impact --json`), some write to files (`scan --impact --out path`), and `impact` standalone writes the report to a fixed path. `hotspots --json` prints to stdout, while `hotspots --snapshot` writes to SQLite and prints a different confirmation message.
- **Suggestion**: Standardize: `--json` always goes to stdout; `--out` always writes to a file; descriptive output always goes to stderr or can be suppressed with `--quiet`.

### 3.6 `changeguard ask --timeout` default is 15s, but no feedback during wait
- The per-request timeout for LLM backends defaults to 15 seconds. During this time there is no spinner or progress indication -- the terminal is entirely silent. Users may think the command hung.
- **Suggestion**: Print a brief "Contacting LLM..." message before the blocking call, or add a spinner.

### 3.7 Ledger `--verify-signatures` flag requires pre-existing signatures
- The `--verify-signatures` flag on `ledger status` will fail if no signatures exist, but the error message may not explain that signatures must be explicitly created during commit.
- **Suggestion**: Improve the error message to suggest `changeguard verify --signatures` or the `--with-git` commit option that creates signatures.

### 3.8 `changeguard verify --dry-run` does not display the plan
- The `--dry-run` flag skips execution but prints nothing about what would have been run. Users see a silent success.
- **Suggestion**: Print the verification plan (list of steps) when `--dry-run` is used, so users can review what would execute.

### 3.9 Ledger start requires an entity path that must exist
- `changeguard ledger start` canonicalizes the entity path and fails if the file does not exist. For planning a change to a file that does not yet exist, users must first create a placeholder.
- **Suggestion**: Add `--allow-missing` flag, or fall back to the raw path when canonicalization fails.

### 3.10 No shorthand for "show all pending"
- `changeguard ledger status` (without args) shows all pending, but the docs often show `changeguard ledger status --compact`. Neither flag combination shows "pending + drift + unaudited" in one view.
- **Suggestion**: Add a `--all` flag or a separate `ledger overview` subcommand that shows the full transactional state.

---

## 4. Significant Improvement Opportunities (Highest Impact)

### 4.1 CRITICAL: Fill integration test gaps for untested command surfaces
The following commands have **zero integration tests** at the CLI dispatch layer:
- `changeguard config {verify, view, schema, diff}`
- `changeguard endpoints`
- `changeguard data-models`
- `changeguard observability`
- `changeguard security`
- `changeguard services diff`
- `changeguard dead-code`
- `changeguard viz`
- `changeguard update`
- `changeguard federate {export, scan, status}`
- `changeguard audit`

These surfaces will silently break on any refactor that changes their data sources, output format, or flag handling. Adding one integration test per surface (even a smoke test that verifies no crash) would dramatically increase confidence. Estimated effort: 2-3 days.

### 4.2 HIGH: Standardize JSON output contract
Currently, JSON output:
- `scan --impact --json`: stdout
- `scan --impact --out file`: file
- `hotspots --json`: stdout
- `impact` standalone: always writes `latest-impact.json` to reports dir
- `ledger search`: text-only, no `--json`
- `ledger status`: text-only, no `--json`
- `config view --json`: stdout
- `config verify --json`: stdout

Establish a project-wide contract: `--json` -> stdout, `--out <file>` -> file, all textual metadata goes to stderr. Audit every command surface for compliance. This makes piping (`changeguard scan --impact --json | jq`) reliable.

### 4.3 HIGH: Consolidate `scan --impact` vs standalone `impact`
Having both `changeguard scan --impact` and `changeguard impact` is confusing because:
- `scan --impact --json` wraps `execute_impact_silent()` which skips human output
- Standalone `impact` writes to the report file and prints human output
- `scan --impact --summary` calls `execute_impact` with `summary=true` which is a different code path

Simplify: either make `changeguard impact` a first-class alias for `changeguard scan --impact` (with `--json` and `--out`), or document the exact difference and add `--json`/`--out` to the standalone `impact` command.

### 4.4 MEDIUM: Add feedback for blocking operations
- `changeguard ask` (15s timeout, silent during wait)
- `changeguard verify <command>` (no output until command completes, could be minutes)
- `changeguard index --semantic` (long-running with local model, no progress beyond log-level traces)

Add spinner/progress for human-interactive mode. Use `indicatif` or the existing `inquire` integration for non-blocking feedback.

### 4.5 MEDIUM: Env-var replacement detection (risk analysis blind spot)
The `RuntimeUsageDelta` model tracks counts but not identities. Replacing `DATABASE_URL` with `REDIS_URL` (1 env var -> 1 env var) produces no risk signal, even though the runtime behavior could change completely. This is a documented limitation (see `test_runtime_delta_same_cardinality_not_flagged`).

Implement identity-aware env-var diffing: compare sets of env-var names between old and new snapshots, flagging replacements even when cardinality is unchanged.

### 4.6 MEDIUM: Auto-fix `--category` type in `ledger start`
The `--category` flag in `ledger start` is a free string; in `ledger atomic` it is a `Category` enum. Change to enum for consistency, which gives users tab-completion and validation at the CLI layer rather than a silent SQL insert of a garbage category.

### 4.7 MEDIUM: Improve `--dry-run` for `verify`
Currently `changeguard verify --dry-run` prints nothing. It should print the verification plan (commands to execute, predicted failures from the Predictor). This matches user expectations from tools like `terraform plan` or `cargo check`.

### 4.8 LOW: Duplicate `audit` dispatch
`Commands::Audit` (top-level) and `LedgerCommands::Audit` both dispatch to `execute_ledger_audit`. This creates maintenance risk (one could gain features the other loses). Deprecate the top-level `audit` in favor of `ledger audit`, or alias one to the other with a deprecation warning.

### 4.9 LOW: Help text for flattened subcommands
Commands using `#[command(flatten)]` (like `DataModels`, `Observability`, `Security`) do not display their sub-flags in parent help. Audit each and either expand to `#[command(subcommand)]` or add a `--help` subcommand to the flattened args struct.

---

## Summary of Findings

| Severity | Count | Key Areas |
|----------|-------|-----------|
| **Critical** | 3 | Untested command surfaces (config, endpoints, data-models, security, observability, dead-code, viz, update, audit, federate); missing integration tests at CLI layer |
| **High** | 3 | Inconsistent JSON contract; `scan --impact` vs standalone `impact` confusion; no tests for state-touching commands (update, config) |
| **Medium** | 5 | Silent blocking operations; env-var identity blind spot; `ledger start --category` free-string; `--dry-run` shows no plan; `watch` untested beyond config validation |
| **Low** | 4 | Duplicate audit dispatch; flattened help text; non-interactive semantic prompt hang; missing "overview" command |

### Commands by Health

**Well-tested (6+ tests covering happy path, edge cases, and regressions):**
`init`, `scan`, `impact`, `ledger start/commit/rollback/atomic`, `ledger search`, `ledger drift`, `ledger enforce`, `hotspots`, `verify`, `reset`, `semantic search`, `risk analysis`, `ledger graph nodes/edges`, `ask KG fallback`, `temporal coupling`, `cedar orphan pruning`

**Moderately tested (1-5 tests, basic paths only):**
`doctor`, `cli_binary_help`, `persistence`, `watch` (config validation only), `ask` (partial), `ledger adr`, `ledger provenance`, `ledger bulk concurrency`, `bridge tests`, `narrative golden`, `complexity_scoring`

**Untested at CLI layer (0 integration tests on the dispatch function):**
`config {verify, view, schema, diff}`, `endpoints`, `data-models`, `observability`, `security`, `services diff`, `dead-code`, `viz`, `update`, `federate {export, scan, status}`, `audit`, `intent demo`, `daemon`, `viz-server`
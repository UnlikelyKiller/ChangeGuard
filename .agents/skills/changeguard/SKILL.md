---
name: changeguard
description: Use this skill when making code edits, reviews, impact/risk analysis, verification planning, drift handling, ledger provenance, or deciding what tests to run. Before meaningful edits, run ChangeGuard scan/impact; after edits, run verification and report unresolved drift or ledger state.
---

# ChangeGuard

Use ChangeGuard as the local safety layer and engineering intelligence engine for code changes. It provides impact analysis, hotspot and temporal-coupling signals, verification planning, and transactional provenance.

## Core Capabilities

- **Search & Discovery**: High-performance regex (Tantivy), precise LSP navigation (SCIP), and conceptual semantic search (local embeddings) with parallel HNSW retrieval.
- **Code Symbol Index**: Tree-sitter parsing of Rust, TypeScript, and Python — extracts every public function, struct, enum, trait, module, and HTTP route into the Knowledge Graph. Queryable via `changeguard search` and `changeguard ask`.
- **Route Extraction**: Detects HTTP routes from Axum, Express, and other frameworks. Stores `method`, `path_pattern`, `handler_name`, `framework`, and confidence score.
- **Call Graph**: Tracks function call relationships (`Direct`, `MethodCall`, `TraitDispatch`, `Dynamic`, `External`) so you can answer "what calls this function?" and "what does this function depend on?".
- **Knowledge Graph**: Durable, billion-edge relational and vector storage (CozoDB-redux/Sled) with native code-aware tokenization (Tree-Sitter). Stores symbols in `project_symbol` table.
- **AI-Brains Bridge**: Exports hotspots, ledger entries, and MADR data to AI-Brains via `changeguard bridge export --hotspots --ledger [--madr] [--stdout]`. AI-Brains nightly pipeline ingests this output as code symbols into recall (T70). Inbound recall uses `changeguard bridge query "<text>"` (IPC with CLI fallback).
- **Impact Analysis**: Deep "blast radius" analysis across 20+ specialized providers (Infra, Contracts, Observability, Temporal).
- **Cryptographic Provenance**: Mathematical proof of intent via Ed25519 signing of every ledger entry. Offline verification via `verify --signatures`.
- **Intent Capture TUI**: Interactive terminal UI for auditing and refining LLM-drafted intent payloads during the git commit process.
- **Real-time Sync**: Incremental Knowledge Graph updates, AST re-parsing, and code-aware symbol indexing via the `watch` command.
- **Predictable Verification**: Bayesian test reordering and CI failure prediction.
- **Documentation Generation**: Export Knowledge Graph data to Markdown/Mermaid passive documentation (`index --export-docs`).
- **Dead Code Detection**: Confidence-based dead code detection blending graph reachability, git activity, and test history (`dead-code` command).
- **Live Visualization**: WebSocket-based Arc Diagram for real-time Knowledge Graph updates (`viz-server`, `viz-server --stop`).
- **Endpoints**: Indexed endpoint graph with auth, schemas, consumers, and owner links. `changeguard endpoints --json` / `--changed` for direct review.
- **Services Diff**: Declared service map with queue/topic/RPC edges and PR-style boundary diff. `changeguard services diff`.
- **Data Models**: Durable data model, table, migration, and compatibility-class relations with impact rules for destructive changes. `changeguard data-models impact --changed`.
- **Config Schema & Diff**: Explicit env var schema metadata (required/secret/owner/provider) and change diff. `changeguard config schema` / `changeguard config diff`.
- **Dependency & Advisory Graph**: Cargo/npm/Python lockfile ingestion with cargo-audit/osv advisory matching. Impact rules for vulnerable dependency introduction.
- **Test Mapping**: Durable test nodes linked to endpoints, symbols, services, and data models. `changeguard verify explain --entity <path>` for entity-scoped test explanation.
- **Observability Graph**: SLO, metric, alert, and signal nodes from OpenSLO YAML. Source-file-backed diff matching. `changeguard observability diff` / `observability coverage`.
- **Hotspot Trends**: Persistent hotspot and temporal coupling snapshots with trend deltas. `changeguard hotspots trend` / `hotspots explain`.
- **Ledger Graph**: Per-transaction entity neighborhood view linking ledger entries to symbols, endpoints, services, ADRs, config keys, and deploy surfaces. `changeguard ledger graph <tx-id>`.
- **Ledger Validator Lifecycle**: Full validator lifecycle with `ledger validator list`, `disable`, `enable`, `remove`, `doctor`, and hook-repair rollback for sidecar/pending mismatches.
- **Security Boundaries**: Cedar policy parsing with cross-surface links (policy→endpoint/service/config_key/deploy_surface/ADR). `changeguard security boundaries` / `security impact --changed`.

## Philosophy: CLI-First Intelligence

ChangeGuard is a **CLI-first** tool and **explicitly rejects MCP/Server/Cloud architecture** for v1. It provides structured, "Gemini-ready" context directly via its CLI outputs. Use ChangeGuard commands as your primary discovery and safety tools.

## Default Workflow

1. Check availability when uncertain:

   ```bash
   changeguard doctor
   ```

2. Check current provenance state:

   ```bash
   changeguard ledger status
   ```

3. Before meaningful code edits, assess impact:

   ```bash
   changeguard scan --impact
   ```

4. Read `.changeguard/reports/latest-impact.json` when it exists. Use it to
   identify risk level, hotspots, temporal couplings, affected symbols, runtime
   dependencies, and verification hints.

5. Make the smallest scoped change that satisfies the task.

6. After edits, run:

   ```bash
   changeguard verify
   ```

   Also run any repo-specific tests needed for the touched files.

7. For final gates, avoid overlapping `cargo`, `nextest`, or `changeguard
   verify` jobs. Parallel read-only inspection is fine, but final verification
   should run sequentially to avoid Windows file-lock and linker contention.

8. Report the outcome: impact/risk signals used, verification run, and any
   unresolved pending transactions, drift, or unavailable ChangeGuard command.

## Code Symbol Queries — Use These First

Before searching the web or reading files manually, query ChangeGuard's symbol index. It knows every public function, struct, route, and call edge in the codebase.

```bash
# Always refresh the index first (incremental, fast)
changeguard index --incremental

# Find a function, struct, or type by name
changeguard search "handleGetUser"
changeguard search "AuthMiddleware"

# Find HTTP routes
changeguard search "POST /auth"
changeguard ask "list all HTTP GET route handlers"

# Find what calls a function
changeguard ask "what calls validateToken"
changeguard ask "show callers of UserRepository::find_by_id"

# Find all public endpoints
changeguard ask "find all Axum route handlers"
changeguard ask "what API endpoints are defined in src/routes"

# Dead code
changeguard dead-code --threshold 0.75
```

These queries work because ChangeGuard indexes:
- Every `pub fn`, `pub struct`, `pub enum`, `pub trait` via tree-sitter
- HTTP route registrations (Axum `Router::route`, Express `app.get`, etc.)
- Function call edges via static analysis
- SCIP-precise symbol navigation from LSP data

Symbols ingested by the bridge become AI-Brains memories (T70) and are returned
by `ai-brains recall "<topic>"` alongside session memories. To verify the
bridge is alive end-to-end, run `ai-brains preflight --summary` and confirm
hotspots and decisions are listed.

## Audit Smoke Tests

When reviewing CLI/config behavior, supplement unit tests with command-level
smoke tests against the current build output, usually `target\debug\changeguard.exe`
on Windows. Prefer focused temporary repositories and verify failure cases as
well as success cases.

Useful checks include:

- JSON mode remains parseable on failure paths (`config verify --json`, invalid
  `config.toml`, invalid `rules.toml`, unknown `--section`).
- Dry-run commands do not create persistent state or perform external probes
  unless that is explicitly part of the dry-run contract.
- Requested vs effective config values are visible when runtime clamping or
  defaults change the final behavior.
- Internal callsites that construct CLI argument structs still populate new
  fields explicitly.

## Repository Configuration

ChangeGuard's `.changeguard/rules.toml` and `.changeguard/config.toml` are
repo-local policy, not portable defaults. When installing or copying this skill
into another repository, review and update:

- `required_verifications`: use commands that actually exist in that repo
  rather than aliases such as `lint`, `test`, or `build` unless the repo defines
  those commands.
- `verify.default_timeout_secs`: set a timeout that fits the repo's slowest
  expected verification command.
- `protected_paths`: keep enforcement scoped to paths that make sense for the
  repository.

If `changeguard verify` fails with "Command not found" or times out while the
same command passes manually, fix the repo-local config before treating it as a
code failure.

## Dependency Alert Workflow

For Dependabot or audit findings:

- Identify whether the vulnerable crate is direct or transitive with
  `cargo tree -i <crate>@<version>`.
- If the vulnerable crate is transitive through a direct dependency, prefer
  upgrading the direct dependency over adding a downstream patch.
- If the vulnerable path enters through a git dependency, verify whether the
  upstream fix is visible to downstream consumers. Workspace-level
  `[patch.crates-io]` entries in the dependency repository are not transitive.
- Record external remediation handoffs in a conductor track when another repo
  owns the durable fix.
- After dependency changes, run focused dependency checks plus `changeguard
  verify`.

## When To Skip

Skip ChangeGuard only for trivial formatting, simple dependency lockfile updates,
binary/media changes, temporary scratch files, or when the user explicitly says
to bypass it.

## If Commands Fail

- If `changeguard` is unavailable, continue with normal repo tools and tell the
  user ChangeGuard signals were unavailable.
- If `ledger status` shows unaudited drift, reconcile or adopt before continuing
  unless the user directs otherwise.
- If `scan --impact` cannot complete, continue cautiously and include the error
  in the final report.
- If a command reports that the index is `[STALE]`, you can append the `--auto-index` flag to commands like `search`, `ask`, `hotspots`, or `dead-code` to automatically refresh it before executing.
- Do not edit `.changeguard/` state files directly.

## Ledger Provenance

For tracked manual edits:

```bash
changeguard ledger start <entity> --category <CAT> --message "Intent"
# edit files
changeguard ledger commit <tx-id> --summary "Done" --reason "Why"
```

For surgical one-command provenance:

```bash
changeguard ledger atomic <entity> --category <CAT> --summary "Task" --reason "Goal"
```

### Git Hook Lifecycle (Milestone O)

ChangeGuard uses a two-phase commit lifecycle to ensure zero phantom records:
1. **`commit-msg`**: Launches the TUI to capture intent. Creates a `PENDING` transaction and a sidecar file.
2. **`post-commit`**: Automatically promotes the `PENDING` transaction to `COMMITTED` once the Git commit is finalized. If the Git commit fails, the record remains pending or is safely rolled back on the next attempt.

### Cryptographic Security

If `intent.require_signing = true` is set in `.changeguard/config.toml`, all ledger entries must be signed by the developer's local Ed25519 key (generated during `init`).

To verify the integrity of the entire ledger:
```bash
changeguard verify --signatures
```
This performs an offline mathematical validation of every record against its signature and public key.

## Publish Hygiene

When asked to push, catch up `main`, or prune branches:

1. Fetch current remote state first:

   ```powershell
   git fetch --all --prune
   git rev-list --left-right --count origin/main...HEAD
   ```

2. If `origin/main` moved, reconcile before staging or pushing. Do not rebase or
   reset over user work without explicit direction.

3. Stage only the intended scope, commit, then push:

   ```powershell
   git push origin main
   ```

   The pre-push hook may run `changeguard verify`; treat that as the authoritative
   publish gate and report its result.

4. Prune conservatively:

   ```powershell
   git remote prune origin --dry-run
   git branch --merged main
   ```

   Delete local branches only when they are listed as merged into `main` and are
   not the active branch. Branch pruning can legitimately be a no-op.

## Reasoning Rules

- If temporal coupling is above 70% for an unchanged file, inspect that file.
- If hotspots are reported, bias verification toward those files first.
- If KG reachability identifies downstream nodes, inspect them before finalizing.
- Treat hooks and CI gates as enforcement. Treat this skill as guidance.

## Maintenance & Upgrades

To keep your ChangeGuard environment synchronized with the latest engine features:

```bash
# Safely migrate repository state (clears indices, preserves ledger)
changeguard update --migrate --force

# Rebuild indices after migration
changeguard index --semantic
```

## Working On ChangeGuard Itself

After changing ChangeGuard source code, you can use the built-in update command to reinstall the global binary:

```bash
changeguard update --binary
```

Alternatively, run manually from the source root:

```bash
cargo install --path .
```

Treat the install step as part of done criteria after ChangeGuard source edits,
before publishing or handing the work back.

## Cross-Model Review Notes

For high-risk diffs, a read-only `codex exec` review can be useful before final
verification. In non-interactive Windows/PowerShell runs, redirect stdin from
`NUL` so the process does not wait for input:

```powershell
cmd /c "codex exec -C ""C:\dev\ChangeGuard"" -s read-only -m gpt-5.4 -o output\review.md ""Review the current diff for regressions. Do not modify files."" < NUL"
```

If the command appears stuck, inspect the output file before waiting longer; the
review may already have written useful findings.

## References

- Command details: `references/commands.md` (includes ledger, impact, dead-code, viz-server, doc generation, watch)
- Install fallback: `references/install.md`
- Architecture/internal notes: `references/internals.md`

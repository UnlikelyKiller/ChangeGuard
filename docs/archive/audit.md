# ChangeGuard Repository Audit

**Date:** 2026-04-19
**Branch:** feat/final-integration
**Audited against:** Plan.md, Engineering.md, breaking.md

---

## 1. Executive Summary

The repo has a functional core loop (init, doctor, scan, impact, verify, ask, watch, reset) with real tree-sitter symbol extraction, SQLite persistence, debounced file watching, and risk scoring. However, the implementation is missing **21 source files**, **8 test files**, the entire `output/`, `util/`, and `verify/` modules, several critical safety features (secret redaction, Gemini modes), and a root `README.md`. There are also engineering-standard violations around silent error suppression, an `unwrap()` in production code, and incomplete git classification.

**Overall verdict: Partially implemented. Core loop works, but significant plan-specified subsystems are absent and several engineering principles are violated.**

---

## 2. File Presence Audit

### 2.1 Source Files: Present and Matching Plan

| Plan Path | Repo Path | Status |
|---|---|---|
| `src/main.rs` | `src/main.rs` | Present |
| `src/cli.rs` | `src/cli.rs` | Present |
| `src/commands/mod.rs` | `src/commands/mod.rs` | Present |
| `src/commands/init.rs` | `src/commands/init.rs` | Present |
| `src/commands/doctor.rs` | `src/commands/doctor.rs` | Present |
| `src/commands/scan.rs` | `src/commands/scan.rs` | Present |
| `src/commands/watch.rs` | `src/commands/watch.rs` | Present |
| `src/commands/impact.rs` | `src/commands/impact.rs` | Present |
| `src/commands/verify.rs` | `src/commands/verify.rs` | Present |
| `src/commands/ask.rs` | `src/commands/ask.rs` | Present |
| `src/commands/reset.rs` | `src/commands/reset.rs` | Present |
| `src/config/mod.rs` | `src/config/mod.rs` | Present |
| `src/config/model.rs` | `src/config/model.rs` | Present |
| `src/config/load.rs` | `src/config/load.rs` | Present |
| `src/config/validate.rs` | `src/config/validate.rs` | Present |
| `src/config/defaults.rs` | `src/config/defaults.rs` | Present |
| `src/platform/mod.rs` | `src/platform/mod.rs` | Present |
| `src/platform/detect.rs` | `src/platform/detect.rs` | Present |
| `src/platform/shell.rs` | `src/platform/shell.rs` | Present |
| `src/platform/paths.rs` | `src/platform/paths.rs` | Present |
| `src/platform/env.rs` | `src/platform/env.rs` | Present |
| `src/state/mod.rs` | `src/state/mod.rs` | Present |
| `src/state/layout.rs` | `src/state/layout.rs` | Present |
| `src/state/migrations.rs` | `src/state/migrations.rs` | Present |
| `src/state/reports.rs` | `src/state/reports.rs` | Present |
| `src/git/mod.rs` | `src/git/mod.rs` | Present |
| `src/git/repo.rs` | `src/git/repo.rs` | Present |
| `src/git/status.rs` | `src/git/status.rs` | Present |
| `src/git/classify.rs` | `src/git/classify.rs` | Present |
| `src/watch/mod.rs` | `src/watch/mod.rs` | Present |
| `src/watch/debounce.rs` | `src/watch/debounce.rs` | Present |
| `src/watch/filters.rs` | `src/watch/filters.rs` | Present |
| `src/watch/batch.rs` | `src/watch/batch.rs` | Present |
| `src/index/mod.rs` | `src/index/mod.rs` | Present |
| `src/index/symbols.rs` | `src/index/symbols.rs` | Present |
| `src/index/languages/mod.rs` | `src/index/languages/mod.rs` | Present |
| `src/index/languages/rust.rs` | `src/index/languages/rust.rs` | Present |
| `src/index/languages/typescript.rs` | `src/index/languages/typescript.rs` | Present |
| `src/index/languages/python.rs` | `src/index/languages/python.rs` | Present |
| `src/impact/mod.rs` | `src/impact/mod.rs` | Present |
| `src/impact/packet.rs` | `src/impact/packet.rs` | Present |
| `src/policy/mod.rs` | `src/policy/mod.rs` | Present |
| `src/policy/rules.rs` | `src/policy/rules.rs` | Present |
| `src/policy/matching.rs` | `src/policy/matching.rs` | Present |
| `src/policy/mode.rs` | `src/policy/mode.rs` | Present |
| `src/policy/protected_paths.rs` | `src/policy/protected_paths.rs` | Present |
| `src/gemini/mod.rs` | `src/gemini/mod.rs` | Present |
| `src/gemini/prompt.rs` | `src/gemini/prompt.rs` | Present |

### 2.2 Source Files: MISSING from Repo

| Plan Path | Plan Phase | Severity |
|---|---|---|
| `src/git/diff.rs` | Phase 5 | HIGH — deleted from working tree; diff summary is a core scan requirement |
| `src/platform/process_policy.rs` | Phase 3/12 | MEDIUM — process execution policy model absent |
| `src/watch/normalize.rs` | Phase 7 | MEDIUM — path normalization for watcher events absent |
| `src/index/references.rs` | Phase 9 | HIGH — import/export relationship extraction missing |
| `src/index/runtime_usage.rs` | Phase 9 | HIGH — env/config usage detection missing |
| `src/index/normalize.rs` | Phase 8/9 | LOW — index normalization absent |
| `src/index/storage.rs` | Phase 8 | MEDIUM — symbol storage API missing |
| `src/impact/score.rs` | Phase 10 | LOW — scoring is consolidated into `analysis.rs` |
| `src/impact/relationships.rs` | Phase 9 | MEDIUM — relationship computation absent |
| `src/impact/reasoning.rs` | Phase 10 | LOW — reasoning is consolidated into `analysis.rs` |
| `src/impact/redact.rs` | Phase 13 | **CRITICAL** — secret redaction for prompts/reports is absent |
| `src/gemini/modes.rs` | Phase 13 | HIGH — `analyze`/`suggest`/`review_patch` modes missing |
| `src/gemini/wrapper.rs` | Phase 13 | HIGH — Gemini wrapper invocation in `gemini/mod.rs` is bare-bones |
| `src/gemini/sanitize.rs` | Phase 13 | **CRITICAL** — prompt sanitization absent; secrets can leak to Gemini |
| `src/output/mod.rs` | Phase 6+ | HIGH — entire output formatting module absent |
| `src/output/json.rs` | Phase 6+ | HIGH — JSON output formatter absent |
| `src/output/table.rs` | Phase 6+ | MEDIUM — table output formatter absent |
| `src/output/diagnostics.rs` | Phase 6+ | MEDIUM — diagnostic output formatter absent |
| `src/output/human.rs` | Phase 6+ | MEDIUM — human-readable output formatter absent |
| `src/util/mod.rs` | Phase 1 | MEDIUM — utility module absent |
| `src/util/fs.rs` | Phase 1 | LOW — fs helpers absent |
| `src/util/hashing.rs` | Phase 1 | LOW — blake3 hashing unused despite being in Cargo.toml |
| `src/util/process.rs` | Phase 12 | MEDIUM — process utilities absent (partially covered by exec/) |
| `src/util/text.rs` | Phase 8 | LOW — text utilities absent |
| `src/util/clock.rs` | Phase 14 | LOW — clock normalization utilities absent |
| `src/verify/mod.rs` | Phase 11 | **CRITICAL** — entire verification planning module absent |
| `src/verify/plan.rs` | Phase 11 | **CRITICAL** — deterministic verification planning missing |
| `src/verify/runner.rs` | Phase 12 | HIGH — verification runner missing (partially covered by exec/) |
| `src/verify/results.rs` | Phase 12 | MEDIUM — structured result persistence missing |
| `src/verify/timeouts.rs` | Phase 12 | LOW — timeout config missing (partially covered by exec/) |
| `src/state/db.rs` | Phase 14 | LOW — replaced by `storage.rs` (acceptable substitution) |
| `src/state/locks.rs` | Phase 14 | LOW — KISS/YAGNI says deferring is acceptable |

### 2.3 Extra Files NOT in Plan

| Repo Path | Assessment |
|---|---|
| `src/lib.rs` | Acceptable — needed for integration test access to crate |
| `src/config/error.rs` | Good addition — aligns with Engineering.md typed error recommendation |
| `src/exec/mod.rs` | Acceptable — fills process execution gap |
| `src/exec/boundary.rs` | Acceptable — timeout/bounded execution (partially covers `verify/runner.rs`) |
| `src/git/ignore.rs` | Good addition — `.gitignore` mutation is Phase 2's core requirement |
| `src/impact/analysis.rs` | Acceptable consolidation — merges `score.rs`, `relationships.rs`, `reasoning.rs` from plan |
| `src/policy/defaults.rs` | Good addition — starter rules content |
| `src/policy/error.rs` | Good addition — typed errors |
| `src/policy/load.rs` | Good addition — rule loading |
| `src/policy/validate.rs` | Good addition — rule validation |
| `src/state/storage.rs` | Acceptable substitution for `db.rs` — clearer naming |
| `src/ui/mod.rs` | Acceptable — minimal UI helpers, but overlaps with missing `output/` module |

### 2.4 Root-Level Files

| Plan Path | Status |
|---|---|
| `Cargo.toml` | Present |
| `Cargo.lock` | Present |
| `README.md` | **MISSING** |
| `LICENSE` | Present |
| `.gitignore` | Present |

### 2.5 Documentation Files

| Plan Path | Status |
|---|---|
| `docs/prd.md` | **MISSING** |
| `docs/implementation-plan.md` | **MISSING** |
| `docs/architecture.md` | **MISSING** |
| `docs/upgrade-notes.md` | **MISSING** |
| `docs/examples/config.toml` | **MISSING** |
| `docs/examples/rules.toml` | **MISSING** |
| `docs/examples/CHANGEGUARD.md` | **MISSING** |

Note: Plan specifies `docs/` (lowercase). Repo uses `Docs/` (capital D). Non-conformant.

### 2.6 CI Files

| Plan Path | Status |
|---|---|
| `.github/workflows/` | **MISSING** — no CI pipeline exists |

### 2.7 Test Files

| Plan Path | Status | Notes |
|---|---|---|
| `tests/cli_init.rs` | Present | Substantive (2 tests) |
| `tests/cli_doctor.rs` | Present | Marginal (1 test, only checks `is_ok()`) |
| `tests/cli_scan.rs` | Present | Substantive (3 tests) |
| `tests/cli_impact.rs` | **MISSING** | Plan specifies; repo has `risk_analysis.rs` instead |
| `tests/cli_verify.rs` | Present | Substantive but Windows-only (hardcodes PowerShell) |
| `tests/state_db.rs` | **MISSING** | Replaced by `persistence.rs` (acceptable) |
| `tests/gitignore_behavior.rs` | **MISSING** | Covered indirectly by inline tests in `git/ignore.rs` |
| `tests/impact_packets.rs` | **MISSING** | Packet tests exist inline in `impact/packet.rs` |
| `tests/verification_plans.rs` | **MISSING** | No verification planning tests exist anywhere |
| `tests/platform_windows.rs` | **MISSING** | No dedicated platform tests |
| `tests/platform_wsl.rs` | **MISSING** | No WSL-specific tests |
| `tests/fixtures/` | **MISSING** | No fixture directory |

---

## 3. Implementation Completeness by Phase

| Phase | Objective | Status | Details |
|---|---|---|---|
| 1 | Bootstrap CLI | **PASS** | `main.rs`, `cli.rs`, all 8 subcommands, logging, diagnostics all working |
| 2 | Repo-Local State + Init | **PASS** | `Layout`, `init`, `.gitignore` updater, starter config/rules all implemented |
| 3 | Doctor + Platform | **PARTIAL** | `doctor`, `detect`, `shell`, `paths`, `env` all work; `process_policy.rs` missing |
| 4 | Config + Rule Loading | **PASS** | Model, load, validate, defaults, error all implemented |
| 5 | Git Scan Foundation | **PARTIAL** | `repo`, `status`, `classify` work; `diff.rs` deleted; `classify` only emits Modified |
| 6 | Basic Impact Packet | **PASS** | Packet schema, generation, JSON report, `impact` command all work |
| 7 | Watch Mode + Debounce | **PARTIAL** | Debounce, filters, batch all work; `normalize.rs` missing; infinite loop in watch command |
| 8 | Language-Aware Indexing | **PARTIAL** | Rust, TypeScript, Python parsers all work; `storage.rs`, `normalize.rs` missing |
| 9 | Relationships + Runtime | **FAIL** | `references.rs`, `runtime_usage.rs`, `normalize.rs` all missing; Phase 9 not implemented |
| 10 | Risk Scoring | **PARTIAL** | Scoring works via `analysis.rs`; not separated into score/relationships/reasoning per plan; no cross-language contract detection |
| 11 | Verification Planning | **FAIL** | Entire `verify/` module missing; no deterministic plan generation |
| 12 | Verification Runner | **PARTIAL** | `exec/boundary.rs` provides timeout/bounded execution; no structured results persistence, no plan integration |
| 13 | Gemini Wrapper | **PARTIAL** | Prompt rendering works; `modes.rs`, `wrapper.rs`, `sanitize.rs` all missing; no mode support; no secret redaction |
| 14 | DB Persistence + Recovery | **PARTIAL** | SQLite via `storage.rs` works; migrations minimal (1 table); no corruption recovery; no DB reset path beyond full `reset --force` |
| 15 | Cross-Platform Hardening | **PARTIAL** | Platform detection, path classification, shell detection work; no line-ending tolerance; no mixed-environment diagnostics beyond WSL mount warning |
| 16 | Documentation + Packaging | **FAIL** | No README, no architecture docs, no CI, no examples directory |

---

## 4. Engineering Standards Compliance (Engineering.md)

### 4.1 SRP Compliance

**Verdict: Partial pass**

Violations:
- **Commands do their own formatting.** Every command handler contains direct `println!` with `owo_colors` formatting. The plan's `output/` module (json, table, diagnostics, human) is meant to own this. Current code mixes presentation logic with command logic.
- **`impact/analysis.rs` consolidates too much.** It combines scoring, relationship assembly, and reasoning into one function. Engineering.md says these should be separated: "relationships computes input facts only, score assigns tier/weights only, reasoning formats human-readable explanations only."
- **`verify` command directly uses `exec::boundary`.** Without the `verify/` module, the command handler is both planning and executing verification, violating SRP.

### 4.2 Idiomatic Rust

**Verdict: Partial pass**

Violations:
- **`unwrap()` in production code**: `src/index/languages/python.rs:38` uses `capture.node.parent().unwrap()` which can panic if a tree-sitter node has no parent.
- **`.expect()` in production code**: `src/gemini/mod.rs:14` and `src/commands/impact.rs:93` use `.expect()` for progress bar styles. These are compile-time-constant strings and unlikely to fail, but the Engineering.md principle is clear: "no `unwrap`/`expect` in production paths."
- **All command functions return `miette::Result<()>`** which is correct.
- **Error types use `thiserror` + `miette::Diagnostic`** which is correct.
- **`Result` propagation with `?`** is used throughout production code (good).

### 4.3 KISS / YAGNI

**Verdict: Mostly pass**

Observations:
- Deferred `state/locks.rs` is acceptable per Engineering.md.
- `process_policy.rs` deferral is acceptable since only one subsystem uses process execution currently.
- The consolidated `impact/analysis.rs` is arguably simpler than the plan's 5-file split, which is fine for current scope.
- Extra dependencies (`owo-colors`, `chrono`, `comfy-table`, `indicatif`, `wait-timeout`) are all actually used — no bloat.

### 4.4 Determinism

**Verdict: Strong pass with gaps**

Passes:
- `ImpactPacket::finalize()` sorts all collections deterministically.
- `get_repo_status()` sorts file changes by path.
- Schema version is pinned (`"v1"`).
- Packet serialization uses `serde_json::to_string_pretty` with `camelCase`.

Gaps:
- **No stable packet field ordering in serialization.** Serde JSON serializes struct fields in declaration order, which is currently stable, but this is implicit rather than guaranteed by a contract.
- **No test fixture normalization for timestamps.** `ImpactPacket::default()` embeds `Utc::now()`, making snapshot comparisons fragile. Tests manually override the timestamp, but there's no normalization helper.
- **Silent suppression of partial data** (see Section 4.5 below).

### 4.5 Error Visibility

**Verdict: Partial pass — significant violations**

Critical violations:

1. **Silent error suppression in `commands/impact.rs`**:
   ```rust
   // Line 38-39: Rules load failure silently discarded
   if let Ok(rules) = crate::policy::load::load_rules(&layout) {
       let _ = crate::impact::analysis::analyze_risk(&mut packet, &rules);
   }
   ```
   Engineering.md says: "invalid config never causes silent fallback without warning." If rules fail to load, the user gets no warning that risk analysis was skipped.

2. **Silent error suppression in `commands/impact.rs`**:
   ```rust
   // Line 56-58: SQLite persistence failure silently discarded
   if let Ok(storage) = crate::state::storage::StorageManager::init(...) {
       let _ = storage.save_packet(&packet);
   }
   ```
   If the DB write fails, the user is told "Wrote impact report" even though the SQLite ledger was not updated. The `ask` command will fail on next invocation because no packet was persisted.

3. **Config validation is a no-op** (`config/validate.rs`):
   ```rust
   pub fn validate_config(_config: &Config) -> Result<()> { Ok(()) }
   ```
   This always succeeds, providing no actual validation.

Good practices:
- Error types include path context (`ConfigError::ReadFailed { path }`, `GitError::RepoDiscoveryFailed { path }`).
- `miette` diagnostics include help text (`StateError::MkdirFailed` suggests checking permissions).
- `ProcessError` distinguishes NotFound vs Timeout vs Failed.

### 4.6 Safety Posture (Plan Section 7)

**Verdict: Partial pass — critical gap**

| Safety Goal | Status |
|---|---|
| No unrestricted AI writes | PASS — wrapper-only invocation |
| No auto-commit behavior | PASS — not implemented |
| Redact likely secrets | **FAIL** — `impact/redact.rs` and `gemini/sanitize.rs` both missing |
| Bounded verification commands | PASS — `exec/boundary.rs` enforces timeouts and output limits |
| Timeout-aware subprocesses | PASS — implemented |
| Local-only state | PASS — no cloud/remote calls except Gemini CLI |

---

## 5. Functional Gaps by Subsystem

### 5.1 Git Subsystem

- **`diff.rs` deleted.** The plan requires diff summary collection for scan reports. Currently, `scan` only shows file-level status (added/modified/deleted), not line-level diff summaries.
- **`classify.rs` is incomplete.** Both `IndexWorktree` and `TreeIndex` changes are classified as `Modified` regardless of actual change type. Added, Deleted, and Renamed changes are not distinguished despite `ChangeType` having those variants. The `is_staged` flag is set based on `TreeIndex` vs `IndexWorktree` but no actual add/delete detection occurs.

### 5.2 Index Subsystem

- **Phase 9 entirely missing.** No import/export summaries, no runtime usage scanners, no env/config extraction. The `ImpactPacket` has no fields for relationships or runtime assumptions.
- **Symbol storage API absent.** Extracted symbols exist only in-memory within the packet. No DB-backed symbol persistence.

### 5.3 Impact Subsystem

- **No secret redaction.** The packet JSON is written to disk and sent to Gemini without any filtering for likely secrets (API keys, tokens, `.env` contents). This is a **security violation** per the plan's Section 7.2.
- **No cross-language contract detection.** The plan's Phase 10 requires elevating cross-language contract changes; `analysis.rs` only checks protected paths, volume, and public symbols.

### 5.4 Verify Subsystem

- **Entire module missing.** The plan specifies a 5-file verify/ module with deterministic plan generation, structured result persistence, and rule-driven command selection. Currently, `commands/verify.rs` takes a single ad-hoc command string and runs it.
- **No verification plan integration.** The `rules.toml` `required_verifications` field is defined but never used by the verify command.

### 5.5 Gemini Subsystem

- **No mode support.** Plan specifies `analyze`, `suggest`, `review_patch` modes. Currently only a single generic prompt exists.
- **No wrapper sophistication.** The `run_query` function in `gemini/mod.rs` pipes raw text to the `gemini` binary. No prompt size limits, no stale packet detection, no error recovery.
- **Windows-specific invocation.** Hardcodes `powershell -Command gemini` on Windows, which is fragile.

### 5.6 Output Subsystem

- **Entirely missing.** All output formatting is inline in command handlers. The plan's `output/` module (json, table, diagnostics, human) is meant to provide structured, consistent formatting. Current approach makes it impossible to add `--format json` output or consistent diagnostic formatting.

### 5.7 Watch Subsystem

- **Infinite loop with no graceful shutdown.** `commands/watch.rs` uses `loop { thread::sleep }` which prevents clean Ctrl+C handling. The plan requires bounded, interruptible watch mode.
- **No batch persistence.** `WatchBatch::save()` exists but is never called from the watch command. The plan requires batches persisted to `.changeguard/state/current-batch.json`.
- **No path normalization.** `watch/normalize.rs` is missing. Watcher events may arrive with inconsistent path casing on Windows.

---

## 6. Dependency Compliance

| Plan Dependency | Cargo.toml Version | Match? | Notes |
|---|---|---|---|
| clap | 4.6.1 (derive) | YES | |
| clap_complete | 4.6.2 | YES | Present but unused (no shell completion generation) |
| clap_mangen | 0.2.31 | YES | Present but unused (no man page generation) |
| serde | 1.0.228 (derive) | YES | |
| serde_json | 1.0 | YES | |
| toml | 1.1.2 | YES | |
| anyhow | 1.0.102 | YES | |
| miette | 7.6.0 (fancy) | YES | |
| thiserror | 2.0 | YES | Breaking changes from breaking.md are handled |
| tracing | 0.1 | YES | |
| tracing-subscriber | 0.3.20 (fmt, env-filter) | YES | |
| notify-debouncer-full | 0.7.0 | YES | |
| ignore | 0.4.25 | YES | Present but unused in source code |
| globset | 0.4.18 | YES | |
| camino | 1.2.2 | YES | Extra `serde1` feature (good) |
| bstr | 1 | YES | |
| rusqlite | 0.39.0 (bundled) | YES | breaking.md: `execute` now checks for tail SQL — code uses single statements, safe |
| rusqlite_migration | 2.5.0 | YES | |
| gix | 0.81.0 | YES | |
| blake3 | 1.8 | YES | Present but **unused** in source code |
| regex | 1.12 | YES | Present but **unused** in source code |
| once_cell | 1.21 | YES | Present but **unused** in source code |
| parking_lot | 0.12 | YES | Used for Mutex in watcher |
| tree-sitter | 0.26.8 | YES | |
| tree-sitter-rust | 0.24.2 | YES | |
| tree-sitter-typescript | 0.23.2 | YES | |
| tree-sitter-python | 0.25.0 | YES | |

**Extra dependencies not in plan:**

| Dependency | Version | Justified? |
|---|---|---|
| owo-colors | 4.3.0 | YES — used for terminal coloring |
| chrono | 0.4.44 (serde) | YES — timestamps in packets and batches |
| wait-timeout | 0.2 | YES — subprocess timeout in exec/boundary |
| comfy-table | 7.1 | YES — table output in scan/impact commands |
| indicatif | 0.17 | YES — progress bars in impact/watch |

**Unused dependencies (present in Cargo.toml but never imported in src/):**

- `ignore` — not imported anywhere
- `blake3` — not imported anywhere
- `regex` — not imported anywhere
- `once_cell` — not imported anywhere
- `clap_complete` — not imported anywhere
- `clap_mangen` — not imported anywhere

---

## 7. Test Coverage Assessment

### 7.1 Test Inventory

| Test File | Tests | Coverage | Issues |
|---|---|---|---|
| `tests/cli_init.rs` | 2 | Good | Duplicated DirGuard helper |
| `tests/cli_doctor.rs` | 1 | Marginal | Only checks `is_ok()`, no output assertions |
| `tests/cli_scan.rs` | 3 | Good | Only `is_ok()` assertions |
| `tests/cli_verify.rs` | 4 | Good | Windows-only (hardcodes PowerShell) |
| `tests/cli_ask.rs` | 1 | Stub | Only tests error path (no packet) |
| `tests/cli_reset.rs` | 5 | Good | Duplicated DirGuard |
| `tests/e2e_flow.rs` | 3 | Good | Skips verify/ask steps; duplicated helpers |
| `tests/persistence.rs` | 1 | Narrow | Only one round-trip scenario |
| `tests/policy_integration.rs` | 1 | Good | Full policy pipeline |
| `tests/risk_analysis.rs` | 3 | Good | Left-behind developer comments |

### 7.2 Missing Test Coverage

- **No black-box CLI tests.** All tests call library functions directly. No test invokes the compiled `changeguard` binary.
- **No verification planning tests.** `verification_plans.rs` from plan is missing; no test for plan generation anywhere.
- **No platform-specific tests.** `platform_windows.rs` and `platform_wsl.rs` from plan are missing.
- **No gitignore behavior tests** in integration test directory (though inline unit tests exist).
- **No impact packet snapshot tests** in integration directory (inline unit tests exist).
- **No fixture directory.** The plan specifies `tests/fixtures/` for shared test data.

### 7.3 Test Quality Issues

- **Duplicated helpers.** `DirGuard` is copy-pasted across 4 test files. Should be in a shared `tests/common/` module.
- **Windows-only verification tests.** `cli_verify.rs` hardcodes PowerShell commands.
- **Shallow assertions.** Several tests only check `result.is_ok()` without inspecting outputs.
- **Left-behind developer comments.** `risk_analysis.rs:53-54` contains inline thought-process comments.

---

## 8. Action Items by Priority

### CRITICAL (Security / Correctness)

1. **Implement `impact/redact.rs` and `gemini/sanitize.rs`.** The absence of secret redaction is a direct safety violation. The ImpactPacket is written to disk and sent to Gemini without any filtering for API keys, tokens, or `.env` contents.
2. **Implement `verify/plan.rs`.** Deterministic verification planning from rules is a core Phase 11 requirement. The `required_verifications` field in rules.toml is defined but never consumed.
3. **Fix silent error suppression in `commands/impact.rs`.** Rules load failure and SQLite write failure must be reported to the user, not silently discarded.

### HIGH (Plan Compliance)

4. **Implement `gemini/modes.rs`.** The plan specifies `analyze`, `suggest`, `review_patch` mode support.
5. **Implement `output/` module.** Extract formatting from command handlers into dedicated formatters (json, table, human, diagnostics).
6. **Implement `index/references.rs` and `index/runtime_usage.rs`.** Phase 9 relationship and runtime usage extraction are entirely missing.
7. **Fix `git/classify.rs`** to distinguish Added/Deleted/Renamed changes instead of always emitting Modified.
8. **Restore or replace `git/diff.rs`.** Diff summary collection is a core scan requirement.
9. **Add `README.md`** at the repo root. A user-facing project must have one.
10. **Add `.github/workflows/`** for CI (fmt, clippy, test at minimum).
11. **Fix `unwrap()` in production code.** `src/index/languages/python.rs:38` must handle the case where `parent()` returns `None`.

### MEDIUM (Engineering Quality)

12. **Fix watch command infinite loop.** Replace `loop { sleep }` with proper signal handling.
13. **Persist watch batches.** Call `WatchBatch::save()` from the watch command.
14. **Add `verify/results.rs`.** Persist structured verification results to `.changeguard/reports/latest-verify.json`.
15. **Remove unused dependencies.** `ignore`, `blake3`, `regex`, `once_cell`, `clap_complete`, `clap_mangen` are all in Cargo.toml but never imported.
16. **Deduplicate test helpers.** Move `DirGuard` to `tests/common/mod.rs`.
17. **Make verification tests cross-platform.** Replace PowerShell-specific commands in `cli_verify.rs`.
18. **Fix `docs/` vs `Docs/` casing.** Plan specifies lowercase.
19. **Expand DB schema.** Currently only `snapshots` table exists; plan specifies `batches`, `changed_files`, `symbols`, `verification_runs`, `verification_results`.
20. **Implement `config/validate.rs`.** Currently a no-op; should validate field constraints.

### LOW (Polish)

21. **Add `watch/normalize.rs`.** Path normalization for watcher events on Windows.
22. **Add `platform/process_policy.rs`.** Process execution policy model.
23. **Add timestamp normalization for tests.** Helper to strip volatile timestamps from packet snapshots.
24. **Add documentation files.** `docs/prd.md`, `docs/architecture.md`, `docs/upgrade-notes.md`, `docs/examples/`.
25. **Add `tests/fixtures/` directory** with shared test data.
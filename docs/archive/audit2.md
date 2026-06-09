# ChangeGuard Repository Audit 2

**Date:** 2026-04-19  
**Audited against:** `docs/Plan.md`, `docs/Engineering.md`, `docs/breaking.md`

## Executive Summary

The repository is materially further along than `docs/audit.md` claims. The current tree includes a real `output/` module, `verify/` plan/results support, Gemini prompt modes, prompt sanitization, impact redaction, `git/diff.rs`, watch-path normalization, CI, a root `README.md`, shared test helpers, and fixture files.

The project is still not fully conformant with the plan. The largest functional gap is `reset`: the plan requires a real reset command and `src/commands/reset.rs`, but the current CLI only prints `"Resetting local state..."` and exits successfully. There are also several structural gaps where planned files are absent or collapsed into other modules, and a few engineering-standard violations around silent fallback behavior and shell-based verification execution.

**Overall verdict:** substantial implementation with a working core loop, but still only **partial compliance** with the full plan.

## Verification Method

I verified the repo by:

- reading `docs/Plan.md`, `docs/Engineering.md`, `docs/breaking.md`
- inventorying the current tree with `rg --files`
- reading the main implementation seams in `commands/`, `git/`, `index/`, `impact/`, `verify/`, `gemini/`, `output/`, `state/`, and `platform/`
- reading the current integration tests and docs
- running:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`

Local verification result: all three commands exited successfully.

## Repository Layout vs Plan

### Root and Docs

| Area | Plan Expectation | Current State | Status |
|---|---|---|---|
| Root files | `Cargo.toml`, `Cargo.lock`, `README.md`, `LICENSE`, `.gitignore` | All present | PASS |
| CI | `.github/workflows/` with verification | `.github/workflows/ci.yml` present | PASS |
| Architecture doc | `docs/architecture.md` | Present | PASS |
| Upgrade notes | `docs/upgrade-notes.md` | Present | PASS |
| Examples | `docs/examples/config.toml`, `rules.toml`, `CHANGEGUARD.md` | All present | PASS |
| PRD | `docs/prd.md` | Missing | FAIL |
| Implementation plan doc path | `docs/implementation-plan.md` | Not present; repo uses `docs/Plan.md` instead | PARTIAL |

### Source Tree

| Subsystem | Plan Expectation | Current State | Status |
|---|---|---|---|
| CLI/bootstrap | `main.rs`, `cli.rs`, command routing, diagnostics | Present and working | PASS |
| Commands | `init`, `doctor`, `scan`, `watch`, `impact`, `verify`, `ask`, `reset` files | All implemented except `src/commands/reset.rs`; `Reset` is a stub in `cli.rs` | FAIL |
| Config | `mod`, `model`, `load`, `validate`, `defaults` | All present; validation is real, not a stub | PASS |
| Platform | `detect`, `shell`, `paths`, `env`, `process_policy` | All present | PASS |
| State | `layout`, `reports`, migrations, DB layer | `storage.rs` is a reasonable `db.rs` substitute; `locks.rs` missing | PARTIAL |
| Git | `repo`, `status`, `diff`, `classify` | All present | PASS |
| Watch | `debounce`, `filters`, `batch`, `normalize` | All present | PASS |
| Index | symbols + language parsers + references + runtime usage | Present; `storage.rs` and `normalize.rs` missing | PARTIAL |
| Impact | packet + scoring/reasoning/relationships + redaction | `analysis.rs` consolidates plan files; `redact.rs` present | PARTIAL |
| Policy | rules, matching, mode, protected paths | Present, plus load/validate/defaults/error helpers | PASS |
| Verify | `mod`, `plan`, `runner`, `results`, `timeouts` | `mod`, `plan`, `results` present; `runner.rs` and `timeouts.rs` absent | PARTIAL |
| Gemini | `modes`, `prompt`, `wrapper`, `sanitize` | `modes`, `prompt`, `sanitize` present; wrapper logic lives in `mod.rs`; `wrapper.rs` missing | PARTIAL |
| Output | `mod`, `json`, `table`, `diagnostics`, `human` | `table.rs` missing; table formatting is embedded in `human.rs` | PARTIAL |
| Util | `fs`, `hashing`, `process`, `text`, `clock` | Only `clock.rs` exists | FAIL |

### Planned Files Missing

The following plan-listed files are still absent:

- `src/commands/reset.rs`
- `src/index/normalize.rs`
- `src/index/storage.rs`
- `src/verify/runner.rs`
- `src/verify/timeouts.rs`
- `src/gemini/wrapper.rs`
- `src/output/table.rs`
- `src/util/fs.rs`
- `src/util/hashing.rs`
- `src/util/process.rs`
- `src/util/text.rs`
- `src/state/locks.rs`
- `docs/prd.md`
- `docs/implementation-plan.md`

### Planned Files Functionally Substituted or Consolidated

These are not exact matches to the plan, but the current substitutes are reasonable:

- `src/state/storage.rs` substitutes for `src/state/db.rs`
- `src/impact/analysis.rs` consolidates `score.rs`, `relationships.rs`, and `reasoning.rs`
- `src/gemini/mod.rs` contains wrapper behavior that the plan expected in `wrapper.rs`
- `src/output/human.rs` includes table rendering that the plan expected to split into `table.rs`

## Phase-by-Phase Audit

| Phase | Plan Goal | Current Status | Notes |
|---|---|---|---|
| 1 | Bootstrap CLI skeleton | PASS | Buildable CLI, subcommands, tracing, diagnostics all exist |
| 2 | Repo-local state and init | PASS | `.changeguard/` layout, starter config/rules, `.gitignore` wiring are implemented |
| 3 | Doctor and platform detection | PASS | Platform, shell, executable detection, WSL path classification all exist |
| 4 | Config and rule loading | PASS | Config loading/validation implemented; rule loading exists |
| 5 | Git scan foundation | PASS | Repo discovery, status collection, staged/unstaged classification, rename/add/delete tests exist |
| 6 | Basic impact packet shell | PASS | Impact packet generation and JSON report writing work |
| 7 | Watch mode and debounce batching | PASS | Debouncing, filtering, normalization, batch persistence, Ctrl+C stop path all exist |
| 8 | Language-aware symbol extraction | PASS | Rust, TypeScript, and Python symbol extraction implemented |
| 9 | Relationships and runtime usage | PASS | Import/export extraction and runtime env/config detection exist |
| 10 | Deterministic risk scoring | PARTIAL | Risk scoring exists, but it is still relatively shallow and not clearly separated by concern |
| 11 | Verification planning | PASS | `verify/plan.rs` exists and is rule-driven |
| 12 | Verification runner | PARTIAL | Verification executes with timeouts and report persistence, but there is no dedicated runner/timeouts module and execution still relies on shell strings |
| 13 | Gemini wrapper integration | PARTIAL | Modes and sanitization exist; wrapper is functional but not split as planned and lacks stronger packet freshness/size controls |
| 14 | DB persistence, migrations, recovery | PARTIAL | SQLite storage and migrations exist, but schema is narrower than the plan and recovery/reset is incomplete |
| 15 | Cross-platform hardening | PARTIAL | Windows/WSL seams exist, but the test matrix is still thin and verify execution is shell-based |
| 16 | Documentation, packaging, release readiness | PARTIAL | README, CI, architecture, upgrade notes, and examples exist; PRD and implementation-plan doc path remain incomplete |

## Engineering Standards Audit

### SRP

**Verdict: partial pass**

Good:

- Major subsystem boundaries exist and are recognizable.
- `platform/`, `state/`, `git/`, `watch/`, `impact/`, `verify/`, and `gemini/` are separated.

Gaps:

- `cli.rs` still owns `Reset` behavior directly instead of delegating to a `commands/reset.rs` module.
- `commands/verify.rs` both plans execution and runs subprocesses instead of delegating to a dedicated runner layer.
- `impact/analysis.rs` still combines multiple concerns the plan intended to split.
- `output/human.rs` currently owns both human formatting and table rendering because `output/table.rs` is missing.

### Idiomatic Rust

**Verdict: pass with minor caveats**

Good:

- Command entry points return `miette::Result<()>`.
- Errors are generally contextual and use `thiserror`/`miette` appropriately.
- I did not find `unwrap()` or `expect()` in production `src/` code paths outside tests. The stale audit is wrong on this point.

Minor caveats:

- Regex initialization uses `expect(...)` in statics. That is common and defensible, but it does technically exceed the strictest reading of the engineering guidance.
- `main.rs` falls back with `unwrap_or_else` for tracing env filter setup, which is acceptable but still a silent fallback.

### KISS / YAGNI

**Verdict: mostly pass**

Good:

- The project did not overbuild `locks.rs`, a plugin system, or deep whole-program analysis.
- A number of plan files were consolidated instead of exploded into premature abstractions.

Gaps:

- The `util/` module is mostly absent rather than intentionally shaped; some responsibilities are split ad hoc across commands and subsystem modules.

### Determinism

**Verdict: partial pass**

Good:

- `get_repo_status()` sorts file changes.
- `verify::plan::build_plan()` sorts and deduplicates commands.
- Packet schema versioning exists.
- `ImpactPacket::finalize()` sorts packet collections.

Gaps:

- `commands/impact.rs` silently drops symbol/import/runtime extraction errors with `.ok().flatten()` instead of recording partial-analysis status.
- `ask`, `watch`, and auto-planned `verify` silently fall back to defaults when config or rules loading fails.
- Verification timestamps are generated inline; there is no test normalization strategy comparable to the packet clock helper.

### Error Visibility

**Verdict: partial pass**

Good:

- `impact` now warns when rules loading or SQLite persistence fails. The stale audit is outdated here.
- Config validation is real and checks debounce, timeout, model, and glob validity.

Findings:

- `load_rules()` does not call `validate_rules()`, so semantically invalid rule patterns can still enter the system until they are ignored downstream.
- `commands/ask.rs` uses `load_config(&layout).unwrap_or_default()`, which suppresses config errors.
- `commands/watch.rs` uses `load_config(&layout).unwrap_or_default()`, which suppresses config errors.
- `commands/verify.rs` uses `load_rules(&layout).unwrap_or_default()` for automatic planning, which suppresses rules-loading failures.
- `commands/impact.rs` suppresses extraction failures for symbols, imports, and runtime usage by discarding errors instead of surfacing partial-analysis diagnostics.

## Functional Findings

### 1. `reset` is not implemented

Severity: HIGH

Evidence:

- `cli.rs` defines a `Reset` subcommand.
- There is no `src/commands/reset.rs`.
- The current branch for `Reset` only prints `"Resetting local state..."` and returns `Ok(())`.

Why it matters:

- The plan explicitly requires a real reset command for rebuilding state and recovering from DB corruption.
- The README advertises `changeguard reset`, but the command does not perform any reset.

### 2. Verification execution still relies on shell strings

Severity: HIGH

Evidence:

- `commands/verify.rs` runs commands with `cmd /C` on Windows and `sh -c` on non-Windows.
- `platform/process_policy.rs` exists but is not applied during verification.

Why it matters:

- The plan explicitly prefers direct process invocation over brittle shell composition.
- Shell-string execution is less deterministic and weaker for policy enforcement.

### 3. Partial analysis failures are silently discarded

Severity: HIGH

Evidence:

- `commands/impact.rs` reads file content and then uses:
  - `parse_symbols(...).ok().flatten()`
  - `extract_import_export(...).ok().flatten()`
  - `extract_runtime_usage(...)`

Why it matters:

- Parser or extractor failure becomes indistinguishable from “no symbols” or “no runtime usage”.
- This conflicts with the determinism and error-visibility guidance in `docs/Engineering.md`.

### 4. Rule validation is not enforced on load

Severity: MEDIUM

Evidence:

- `policy/validate.rs` exists.
- `policy/load.rs` parses TOML but never calls `validate_rules()`.

Why it matters:

- Invalid glob patterns are supposed to fail clearly.
- Current behavior allows invalid rules to survive until later code ignores or sidesteps them.

### 5. Diff support exists but is not integrated into scan reporting

Severity: MEDIUM

Evidence:

- `src/git/diff.rs` exists and shells out to `git diff`.
- `commands/scan.rs` only prints the status summary and does not surface diff summaries.

Why it matters:

- The plan calls for diff summary collection as part of scan/report foundations.
- The implementation is only partial: helper exists, user-facing scan output does not expose it.

### 6. SQLite persistence is only partially aligned with the planned schema

Severity: MEDIUM

Good:

- Migrations create `snapshots`, `batches`, `changed_files`, `verification_runs`, and `verification_results`.

Gap:

- There is no dedicated `symbols` table.
- There is no explicit recovery/reset path tied to corruption handling because `reset` is unimplemented.

### 7. `util/` remains largely unimplemented

Severity: MEDIUM

Evidence:

- Only `src/util/clock.rs` exists.
- Planned helpers for fs, hashing, process, and text are absent.

Why it matters:

- This is mostly a structure gap, not a runtime failure.
- It does mean some cross-cutting concerns are still scattered.

## Testing Audit

### What Exists

- Integration tests for `init`, `doctor`, `scan`, `impact`, `verify`, and `ask` error-path handling
- persistence and policy integration tests
- shared test helpers in `tests/common/mod.rs`
- fixture files in `tests/fixtures/`

### Gaps vs Plan

| Planned Test Artifact | Current State | Status |
|---|---|---|
| `tests/cli_init.rs` | Present | PASS |
| `tests/cli_doctor.rs` | Present | PASS |
| `tests/cli_scan.rs` | Present | PASS |
| `tests/cli_impact.rs` | Present | PASS |
| `tests/cli_verify.rs` | Present | PASS |
| `tests/state_db.rs` | Not present; `tests/persistence.rs` covers part of this | PARTIAL |
| `tests/gitignore_behavior.rs` | Not present; coverage is inline in `src/git/ignore.rs` | PARTIAL |
| `tests/impact_packets.rs` | Not present; coverage is inline in `src/impact/packet.rs` | PARTIAL |
| `tests/verification_plans.rs` | Not present; coverage is inline in `src/verify/plan.rs` | PARTIAL |
| `tests/platform_windows.rs` | Missing | FAIL |
| `tests/platform_wsl.rs` | Missing | FAIL |
| `tests/fixtures/` | Present | PASS |

Additional quality notes:

- Several integration tests assert only `is_ok()` and do not inspect user-facing output.
- There are still no black-box tests that invoke the built CLI binary as a subprocess.

## Dependency and CI Audit

### Dependency Alignment

The repo uses the core planned dependency set for the implemented features, but not the full baseline from the plan.

Notable differences:

- present: `clap`, `serde`, `serde_json`, `toml`, `anyhow`, `miette`, `thiserror`, `tracing`, `notify-debouncer-full`, `globset`, `camino`, `rusqlite`, `rusqlite_migration`, `gix`, `regex`, `parking_lot`, tree-sitter crates
- absent from current `Cargo.toml`: `clap_complete`, `clap_mangen`, `ignore`, `blake3`, `once_cell`, `bstr`
- extra but justified: `owo-colors`, `chrono`, `wait-timeout`, `comfy-table`, `indicatif`, `ctrlc`

This is acceptable where the corresponding plan features are not implemented yet, but it means the repo is not a strict file-for-file realization of the dependency baseline.

### CI Alignment

`.github/workflows/ci.yml` is present and runs:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test -j 1 -- --test-threads=1`
- `cargo audit`

Gap:

- the plan also calls for `cargo deny`; CI does not run it yet.

## Recommended Priority Order

1. Implement a real `reset` command in `src/commands/reset.rs` and wire actual state cleanup/recovery behavior into the CLI.
2. Stop suppressing config/rules failures in `ask`, `watch`, and `verify`; surface actionable diagnostics instead of silently falling back.
3. Record partial-analysis failures in impact generation instead of collapsing them into missing data.
4. Enforce rule validation during load by calling `validate_rules()` from `policy/load.rs`.
5. Refactor verification execution away from shell-string composition, or at minimum enforce `process_policy` on the current path.
6. Complete the missing structural plan files where they still matter: `verify/runner.rs`, `verify/timeouts.rs`, `gemini/wrapper.rs`, `output/table.rs`, and the remaining `util/` helpers.
7. Add the missing docs artifacts: `docs/prd.md` and either `docs/implementation-plan.md` or a documented alias from `docs/Plan.md`.
8. Expand test coverage for reset, dedicated Windows/WSL behavior, and black-box CLI invocation.

## Final Verdict

Compared to the plan, the repo is no longer in the “core loop only” state described by `docs/audit.md`. It already implements most of phases 1 through 13 in some form and has meaningful phase 14 through 16 work in place.

It still does **not** fully satisfy the plan because:

- `reset` is missing as real functionality
- several planned files remain absent
- some engineered boundaries are still collapsed
- error visibility is weakened by multiple silent fallbacks
- cross-platform/process safety for verification is not hardened enough

The right summary is: **working and substantial, but not yet fully plan-complete or engineering-complete**.

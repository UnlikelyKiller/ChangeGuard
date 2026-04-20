# ChangeGuard Repository Audit 4

**Date:** 2026-04-20  
**Audited against:** `docs/Plan.md`, `docs/Plan-Phase2.md`, `docs/Engineering.md`, `docs/audit3.md`, `conductor/track30` through `conductor/track35`

## Executive Summary

The implementor's remediation work resolves some of the previous audit's concrete defects, but the claim that Tracks 30-35 are complete is not supported.

Notable improvements:

- `cargo test` and `cargo test --all-features` now pass.
- `cargo check`, `cargo check --all-features`, and `cargo build --no-default-features` pass.
- Impact persistence now redacts before SQLite save.
- Temporal traversal now has first-parent default wiring and an `all_parents` option.
- Structural prediction, federated dependency storage, narrative token truncation, and daemon scaffolding have been added.

Blocking failures remain:

- `cargo fmt --check` fails.
- `cargo clippy --all-targets --all-features -- -D warnings` fails.
- Track 35 daemon is still not functionally complete: hover and CodeLens return `None`, diagnostics only replay cached packets, broken-stdin behavior is a placeholder, and there are no daemon lifecycle tests.
- Track 31 is incomplete: hotspot row errors are still silently dropped, hotspot tests are missing, complexity tests remain weak, TypeScript/edge tests are missing, and the required arborist spike documentation is absent.
- Track 32 is partial: structural prediction is heuristic and only uses historical packet rows, temporal analysis is not recomputed in `verify`, predictor warnings are not persisted into the verification report, and placeholder thought-process comments remain in tests.
- Track 33 is partial: federation has better schema/version/path handling and dependency persistence, but auto-detection in `impact` is still absent and dependency discovery only scans previously changed files from the latest packet.
- Track 34 is partial: token truncation and fallback artifacts are wired, but prompt construction embeds the narrative prompt as the "Question" inside the generic impact prompt, and several error/fallback paths silently ignore write failures.

**Overall verdict:** Phase 2 is improved but still not complete. Track 30 fails outright because required gates are not green. Tracks 31-35 remain partial.

## Verification Results

Local commands run during this audit:

| Command | Result |
|---|---|
| `cargo check` | PASS |
| `cargo check --all-features` | PASS |
| `cargo build --no-default-features` | PASS |
| `cargo test -j 1 -- --test-threads=1` | PASS |
| `cargo test --all-features -j 1 -- --test-threads=1` | PASS |
| `cargo fmt --check` | **FAIL** |
| `cargo clippy --all-targets --all-features -- -D warnings` | **FAIL** |

`cargo fmt --check` reports formatting diffs in the new remediation files, including:

- `src/cli.rs`
- `src/commands/ask.rs`
- `src/commands/daemon.rs`
- `src/commands/federate.rs`
- `src/commands/hotspots.rs`
- `src/commands/impact.rs`
- `src/commands/verify.rs`
- `src/daemon/*`
- `src/federated/impact.rs`
- `src/gemini/wrapper.rs`
- `src/impact/hotspots.rs`
- `src/impact/temporal.rs`
- `src/verify/predict.rs`
- `tests/federated_discovery.rs`
- `tests/narrative_golden.rs`
- `tests/narrative_prompt.rs`
- `tests/predictor.rs`
- `tests/temporal_coupling.rs`

`cargo clippy --all-targets --all-features -- -D warnings` fails with:

- `src/commands/ask.rs`: collapsible nested `if`.
- `src/impact/hotspots.rs`: collapsible nested `if`.
- `src/impact/temporal.rs`: `and_then(|x| Ok(y))` should be `map`.
- `src/daemon/state.rs`: collapsible nested `if`.

## Audit3 Resolution Status

| Audit3 item | Status | Notes |
|---|---|---|
| Green verification gates | **Not resolved** | Tests now pass, but `fmt` and all-feature clippy fail. |
| Redact before SQLite persistence | Resolved | `src/commands/impact.rs` finalizes/redacts before `storage.save_packet()`. |
| Production unwraps in impact/hotspots | Mostly resolved | Targeted command/hotspot production unwraps are gone; tests still contain unwraps, which is acceptable. |
| Temporal first-parent default | Mostly resolved | `first_parent_only()` is used by default; no real git fixture test was found. |
| Complexity degradation | Partial | `ast_incomplete` and `complexity_capped` exist, but unsupported-language modeling and tests are incomplete. |
| Deterministic hotspot sorting | Partial | Path tiebreaker exists, but row errors are still dropped and no hotspot tests were found. |
| Structural prediction | Partial | Implemented as packet-history import heuristics, not current index/repo reverse-dependency analysis. |
| Predictor placeholder comments | Partial | Production comments improved, but implementor thought-process comments remain in `tests/predictor.rs`. |
| Federated impact placeholder | Partial | Generic warning was replaced, but workflow still depends on explicit `federate scan` and latest packet changed files. |
| Schema validation/path confinement | Partial | Version validation, catch-unwind, cap, and canonical parent checks exist; tests are thin and auto-detection remains missing. |
| Narrative token budgeting | Partial | `truncate_for_context(409600)` is called, but the narrative prompt is nested into generic prompt construction. |
| Gemini failure fallback | Partial | A fallback artifact is attempted, but write errors are ignored. |
| LSP daemon | Partial/fail | A tower-lsp scaffold exists, but Hover/CodeLens are stubbed and lifecycle requirements are incomplete. |

## Track 30: Foundation & Safety Remediation

**Status: fail**

Resolved:

- Plain and all-feature tests pass.
- `src/commands/impact.rs` now finalizes and redacts the packet before SQLite persistence.
- The specific audit3 production unwraps in `src/commands/impact.rs`, `src/commands/hotspots.rs`, and `src/impact/hotspots.rs` are mostly removed.

Remaining blockers:

1. **Required gates are not green.**  
   Track 30 explicitly requires `cargo fmt --check` and all-feature clippy to pass. Both fail.

2. **All-feature clippy failures are in remediation code.**  
   The failures are not inherited noise; they are in `src/commands/ask.rs`, `src/impact/hotspots.rs`, `src/impact/temporal.rs`, and `src/daemon/state.rs`.

3. **Silent failure paths remain.**  
   `src/commands/verify.rs` uses `.ok()` and `.unwrap_or_default()` around SQLite initialization, packet load, and history load. Prediction and persistence failures are logged, not reported in the verification JSON.

## Track 31: Intelligence & Determinism Hardening

**Status: partial**

Resolved:

- `src/impact/temporal.rs` uses `first_parent_only()` unless `all_parents` is enabled.
- Threshold comparison is now strict `>`.
- `Impact` and `Hotspots` expose `all_parents`.
- `FileComplexity` now has `ast_incomplete` and `complexity_capped`.
- `NativeComplexityScorer` implements `Default`.
- Hotspot scoring now uses normalized multiplication and path tiebreaking.
- `commands/hotspots.rs` delegates to the shared `impact::hotspots::calculate_hotspots()`.

Remaining gaps:

1. **No documented arborist spike decision.**  
   `rg "arborist"` finds plan/audit references only. No ADR or dedicated spike result document exists.

2. **Complexity unsupported-language model is still not represented.**  
   The scorer API still accepts `Language`, which only covers supported languages. There is no `Complexity::NotApplicable` or equivalent result type.

3. **Complexity tests do not meet the plan.**  
   `tests/complexity_scoring.rs` still asserts `> 1` for complex cases and includes exploratory comments. No TypeScript test, syntax-error test, unsupported-language test, or large-file cap test was found.

4. **No real temporal git fixture test was found.**  
   `tests/temporal_coupling.rs` still uses an in-memory mock provider only.

5. **Hotspot row errors are still silently dropped.**  
   `src/impact/hotspots.rs` still uses `.filter_map(|res| res.ok())`, hiding malformed SQLite rows.

6. **No dedicated hotspot test was found.**  
   There is no `tests/hotspot_ranking.rs` or equivalent coverage for normalized multiplication, JSON schema, filters, empty history, all-zero complexity, or tie-breaking.

## Track 32: Predictive Verification Completion

**Status: partial**

Resolved:

- `src/verify/predict.rs` now combines temporal and structural predictions.
- `PredictionResult` carries warnings.
- Verification plan deduplication now merges descriptions for traceability.
- Basic predictor tests were added.

Remaining gaps:

1. **Structural prediction is not based on current repo/index reverse-dependency analysis.**  
   `Predictor::predict()` scans imports from historical `ImpactPacket` rows, not the current source tree or symbol index. This can miss files that have never appeared in prior packets and can use stale imports.

2. **`verify` does not recompute temporal analysis before plan construction.**  
   `src/commands/verify.rs` reads the latest packet and historical packets from SQLite. It does not run the temporal engine if the packet lacks temporal data.

3. **Prediction warnings are not persisted into the verification report.**  
   Warnings are sent to `tracing::warn!` only. `latest-verify.json` has no prediction diagnostic field, so the fallback is not user-visible in the report.

4. **Storage/history failures are silently degraded.**  
   `StorageManager::init(...).ok()`, `get_latest_packet().ok()`, `get_all_packets().ok()`, and `unwrap_or_default()` hide why prediction cannot run.

5. **Predictor tests contain unfinished reasoning comments.**  
   `tests/predictor.rs` includes comments such as "Wait, PredictedFile Eq/Ord includes reason" and "Let's verify intended behavior", which should not remain in committed verification tests.

## Track 33: Federated Intelligence Completion

**Status: partial**

Resolved:

- `FederatedSchema::validate()` rejects unsupported schema versions.
- `FederatedScanner` uses `catch_unwind`, `symlink_metadata`, canonical parent checks, deterministic warnings, and a default sibling cap of 20.
- `federate scan` persists dependency edges.
- `check_cross_repo_impact()` now checks stored dependencies against sibling schemas and reports removed interfaces.
- `unwrap_or("unknown")` was removed from export.

Remaining gaps:

1. **Federation is still not automatic in `scan` or `impact`.**  
   The Phase 2 workflow requires scan/impact to automatically detect sibling schemas. Current impact only checks already-stored federated links; users must run `changeguard federate scan` first.

2. **Dependency discovery only searches changed files in the latest packet.**  
   `FederatedScanner::discover_dependencies()` loops over `local_packet.changes`. It does not scan the local repo or symbol index broadly, so stable dependencies outside the latest changed-file set are missed.

3. **Export redaction is ad hoc.**  
   `src/commands/federate.rs` includes a "For now" comment and manually filters symbol names containing strings like `KEY` or `TOKEN`. It does not reuse the project's secret redaction model.

4. **Path confinement tests are weak.**  
   The added test does not create a malicious schema path or symlink escape with a schema and only asserts no siblings in a nested layout.

5. **Schema validation is minimal.**  
   Version validation exists, but there is no strict JSON schema or field-level validation beyond serde structure and version.

## Track 34: Narrative Reporting Completion

**Status: partial**

Resolved:

- `--narrative` is present on `ask`.
- Narrative mode no longer requires query text to equal `summary`.
- `truncate_for_context(409600)` is called before prompt construction.
- The required truncation annotation is appended when truncation occurs.
- `gemini analyze` is invoked.
- Missing Gemini CLI returns the requested actionable message.
- A deterministic narrative prompt golden test exists.
- Gemini failure attempts to save `.changeguard/reports/fallback-impact.json`.

Remaining gaps:

1. **Narrative prompt construction is awkward and likely duplicated.**  
   `commands/ask.rs` generates a full narrative prompt, then passes it as the `Question:` into `build_user_prompt()`, which also embeds the full impact packet JSON. The Phase 2 spec asks for structured narrative prompt input, not a narrative prompt nested inside a generic prompt template.

2. **Fallback write failures are ignored.**  
   `create_dir_all(...).ok()`, `serde_json::to_string_pretty(...).ok()` via `if let`, and `std::fs::write(...).is_ok()` mean users may receive only the Gemini error without knowing fallback creation failed.

3. **Token budgeting is character-count based only.**  
   This matches Track 34's remediation approximation, but it is still not a real token estimator. The Phase 2 plan asked for a token budget estimator against 80% of the configured Gemini context window.

4. **No wrapper-level truncation protection exists.**  
   The only truncation is in `ask`; `gemini::wrapper::run_query()` accepts any prompt size.

## Track 35: LSP Daemon Resolution

**Status: fail/partial**

Resolved:

- `tokio` and `tower-lsp-server` are optional behind the `daemon` feature.
- `src/daemon/` exists with `server`, `handlers`, `lifecycle`, and `state` modules.
- `commands/daemon.rs` builds a Tokio runtime with two worker threads.
- The server advertises text sync, Hover, and CodeLens capabilities.
- PID file setup and stale process checks exist.

Remaining critical gaps:

1. **Hover and CodeLens are stubs.**  
   `src/daemon/handlers.rs` returns `Ok(None)` for both `on_hover()` and `on_code_lens()`. This does not meet the Track 35 or Phase 21 requirements.

2. **The file still contains explicit placeholder comments.**  
   `src/daemon/handlers.rs` says "In a real implementation..." and `src/daemon/lifecycle.rs` says "For now..." about stdin handling.

3. **Diagnostics do not perform real-time analysis.**  
   `trigger_analysis()` only reads the latest stored packet and publishes diagnostics for the matching path. It does not run or schedule analysis for the opened/changed/saved document.

4. **Broken stdin self-termination is not implemented.**  
   `check_stdin_alive()` always returns `true` and is not wired into the server lifecycle.

5. **Read-only SQLite handling is questionable.**  
   `ReadOnlyStorage::get_connection()` opens the DB with `SQLITE_OPEN_READ_ONLY`, then executes `PRAGMA journal_mode=WAL;`, which may require write capability and can fail in read-only mode.

6. **`data_stale` is not surfaced in LSP outputs.**  
   `ReadOnlyStorage` returns `QueryResult { data_stale }`, but handlers ignore the flag.

7. **No daemon lifecycle tests were found.**  
   `rg --files tests | rg "daemon"` returns no tests. The required PID lifecycle, stale cleanup, shutdown, SQLite contention, URI normalization, Hover, and CodeLens tests are missing.

## Engineering Standards Findings

### 1. Green gates are still not restored

Severity: CRITICAL

Track 30 requires green formatting and clippy. The repository currently fails both.

### 2. Placeholder comments remain in production code

Severity: HIGH

Examples:

- `src/daemon/handlers.rs`: "In a real implementation..."
- `src/daemon/lifecycle.rs`: "For now..."
- `src/commands/federate.rs`: "For now, we'll redact..."

### 3. Silent degradation remains

Severity: HIGH

Examples:

- `src/impact/hotspots.rs` drops row errors with `.filter_map(|res| res.ok())`.
- `src/commands/verify.rs` hides packet/history/storage failures with `.ok()` and `unwrap_or_default()`.
- `src/commands/ask.rs` ignores fallback artifact write failures.
- LSP handlers ignore `data_stale`.

### 4. Tests are present but not sufficient

Severity: HIGH

New tests cover some happy paths, but the plan's required edge tests are still missing for hotspots, daemon lifecycle, daemon SQLite contention, TypeScript complexity, syntax-error complexity, unsupported complexity, large-file complexity cap, real temporal git fixtures, and strict federated path/schema failures.

### 5. SRP and KISS issues remain

Severity: MEDIUM

Federated dependency discovery reads file contents from the scanner, tying discovery, schema parsing, and local-code matching into one module. The daemon advertises full LSP capabilities while handlers return empty results for two of the three core features.

## Priority Action Items

Critical:

1. Run `cargo fmt` and fix all all-feature clippy failures.
2. Finish or downgrade Track 35: implement real Hover and CodeLens, remove placeholders, wire stale-data diagnostics, implement broken-stdin behavior, and add daemon lifecycle/contention tests.
3. Stop silently dropping hotspot SQLite row errors.
4. Make prediction degradation visible in `latest-verify.json`, not only tracing logs.

High:

5. Add the missing hotspot scoring/filter/JSON/tie tests.
6. Add real temporal git fixture tests.
7. Add TypeScript, syntax-error, unsupported-language, and large-file complexity tests.
8. Document the `arborist-metrics` spike decision.
9. Rework federation dependency discovery to scan current repo/index data, not only latest changed files.
10. Make federation discovery automatic in `scan` or `impact`, or document why Phase 2 accepts explicit `federate scan`.
11. Replace ad hoc federated export redaction with the shared redaction model.
12. Rework narrative prompt construction so narrative mode uses one deterministic structured prompt rather than nesting that prompt into the generic `Question:` field.

## Final Verdict

The repository has progressed since `docs/audit3.md`, especially around tests, redaction-before-save, first-parent temporal traversal, and scaffolding for prediction/federation/narrative/daemon features. It is still not Phase 2 complete.

The strongest objective blocker is that the required gates still fail: `cargo fmt --check` and all-feature clippy are red. Beyond that, the daemon track remains substantively incomplete, hotspot and complexity edge coverage are missing, prediction and federation still rely on stale or narrow packet data, and several failure paths remain invisible to users.

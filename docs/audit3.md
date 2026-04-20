# ChangeGuard Repository Audit 3

**Date:** 2026-04-20  
**Audited against:** `docs/Plan-Phase2.md`, `docs/Engineering.md`, `conductor/track23` through `conductor/track29`

## Executive Summary

The implementor's claim that Phase 2 is complete is not supported by the current repository.

The source tree contains many Phase 2 files and some useful implementation work: temporal coupling exists, native complexity scoring exists, hotspots have a command path, predictive verification has a plan expansion hook, federation has export/scan/status scaffolding, and Gemini narrative mode exists.

However, Phase 2 is **not engineering-complete or spec-complete**:

- `cargo fmt --check` fails.
- `cargo test` fails.
- `cargo test --all-features` fails.
- `cargo clippy --all-targets --all-features -- -D warnings` fails.
- Track 27 daemon is effectively not implemented as an LSP daemon.
- Track 26 predictive verification contains implementor thought-process comments and does not implement structural prediction.
- Track 28 federated impact is a generic placeholder warning, not cross-repo impact resolution.
- Track 29 token budgeting/truncation is not wired into Gemini execution.
- Several Phase 2 commands silently drop row-level errors or use unstable/non-deterministic sorting paths.

**Overall verdict:** Phase 2 is **partial**. Tracks 23 and 24 are the most substantive. Tracks 25, 26, 28, and 29 are partial. Track 27 fails the plan.

## Verification Method

I verified the repository by:

- reading `docs/Plan-Phase2.md`
- reading `conductor/track23/spec.md` through `conductor/track29/spec.md`
- reading `conductor/track23/plan.md` through `conductor/track29/plan.md`
- reading the Phase 2 implementation in `src/impact`, `src/index`, `src/verify`, `src/output`, `src/federated`, `src/gemini`, `src/commands`, `src/state`, and `src/cli.rs`
- checking for placeholders, production `unwrap`/`expect`, and incomplete comments
- running the standard verification gates

Local verification results:

| Command | Result |
|---|---|
| `cargo check` | PASS |
| `cargo check --all-features` | PASS |
| `cargo build --no-default-features` | PASS |
| `cargo fmt --check` | **FAIL** |
| `cargo test -j 1 -- --test-threads=1` | **FAIL** |
| `cargo test --all-features -j 1 -- --test-threads=1` | **FAIL** |
| `cargo clippy --all-targets --all-features -- -D warnings` | **FAIL** |

The test failure is caused by a stale test calling `execute_watch(100)` even though `execute_watch` now takes `(interval_ms, json_output)`.

The clippy/all-targets failure includes:

- missing `ChangedFile` import in `src/output/lsp.rs` tests
- `NativeComplexityScorer::new()` without `Default`
- collapsible `if` warnings elevated to errors

## Track Status Summary

| Track | Goal | Verdict | Summary |
|---|---|---|---|
| 23 | Temporal Intelligence | PARTIAL PASS | Real gix-based history crawler and affinity scoring exist, but traversal is not first-parent, fixture coverage is thin, and edge cases are incomplete. |
| 24 | Complexity Indexing | PARTIAL PASS | Native scorer and DB persistence exist, but graceful degradation, unsupported-language model, large-file cap, and documented arborist spike are missing. |
| 25 | Hotspots | PARTIAL | Command and scoring exist, but output/options/scoring do not meet the plan, and error handling/determinism have gaps. |
| 26 | Predictive Verification | FAIL/PARTIAL | Plan hook exists, but structural prediction is not implemented and the file contains placeholder reasoning comments. |
| 27 | LSP-Lite Daemon | FAIL | No `src/daemon/` implementation exists; daemon command reuses watch JSON output and is not an LSP server. |
| 28 | Federated Intelligence | PARTIAL | Export/scan/status scaffolding exists, but impact resolution is placeholder-like and security/schema validation is incomplete. |
| 29 | Advanced Narrative Reporting | PARTIAL | Narrative mode exists, but token budgeting/truncation is unused and Gemini failure fallback is incomplete. |

## Track 23: Temporal Intelligence

**Status: partial pass**

What is implemented:

- `src/impact/temporal.rs` exists.
- `GixHistoryProvider` uses `gix`.
- Shallow clone detection exists via `repo.is_shallow()`.
- Merge commits and giant commits are skipped in coupling calculation.
- `TemporalConfig` has `max_commits`, `max_files_per_commit`, and `coupling_threshold`.
- Couplings are sorted before return.
- Impact generation attempts temporal analysis and surfaces a warning when it fails.

Gaps:

1. **Traversal is not first-parent as required.**  
   `src/impact/temporal.rs` uses `Sorting::BreadthFirst`; the spec requires first-parent traversal by default.

2. **The requested `HistoryCrawl` API is not implemented.**  
   The code uses `HistoryProvider` and `TemporalEngine`. This is an acceptable shape change only if behavior is complete, but behavior is still partial.

3. **Threshold semantics differ from the spec.**  
   The spec says coupling is `>75%`. The implementation uses `>= coupling_threshold`.

4. **No real git fixture test was found.**  
   `tests/temporal_coupling.rs` uses an in-memory mock provider. The track plan explicitly requires a synthetic git repo fixture test.

5. **No explicit all-parents opt-in flag exists.**  
   `docs/Plan-Phase2.md` says first-parent is default and `--all-parents` is opt-in. No such CLI/config path exists.

6. **Partial failure handling for unparseable commits is not implemented.**  
   The plan calls for partial failures to be annotated rather than fatal. Current traversal maps most `gix` errors directly into `GitError::MetadataError`.

## Track 24: Complexity Indexing

**Status: partial pass**

What is implemented:

- `src/index/metrics.rs` exists.
- `ComplexityScorer` trait exists.
- `NativeComplexityScorer` computes rough cyclomatic and cognitive scores using tree-sitter.
- `symbols` table includes `cognitive_complexity` and `cyclomatic_complexity`.
- `persist_symbols()` writes complexity scores to SQLite.
- `commands/impact.rs` invokes complexity scoring during changed-file analysis.
- Basic Rust and Python tests exist in `tests/complexity_scoring.rs`.

Gaps:

1. **No documented `arborist-metrics` spike result.**  
   The plan requires running and documenting the spike before choosing native fallback. I found no evidence of that decision record.

2. **Syntax-error degradation is not modeled.**  
   The spec requires partial metrics with `ast_incomplete: true`. `FileComplexity` has no `ast_incomplete` field and `score_file()` does not inspect `tree.root_node().has_error()`.

3. **Unsupported-language behavior is not represented.**  
   The spec requires `Complexity::NotApplicable` or equivalent. The API accepts only `Language`, so unsupported languages cannot be represented by the scorer.

4. **Large-file complexity cap is missing.**  
   `docs/Plan-Phase2.md` requires a cap/annotation for files over 10,000 lines. No cap or `complexity_capped` field exists.

5. **Production `unwrap()` remains in the complexity integration path.**  
   `src/commands/impact.rs` calls `Utf8Path::from_path(relative_path).unwrap()` while scoring. That violates the Phase 2 no-`unwrap()` rule.

6. **TypeScript support is likely incomplete.**  
   The scorer matches `method_declaration` and `arrow_function`, but not common TypeScript tree-sitter forms like `method_definition`. There is no TypeScript complexity integration test.

7. **The test quality is weak.**  
   `tests/complexity_scoring.rs` asserts scores are greater than 1 rather than checking hand-calculated golden values. The test contains exploratory comments rather than a firm expected model.

## Track 25: Hotspot Identification

**Status: partial**

What is implemented:

- `src/commands/hotspots.rs` exists.
- `src/impact/hotspots.rs` exists.
- Hotspot scoring combines file change frequency with stored complexity.
- Human table output exists through `output::human`.
- Hotspots are added to impact packets when impact generation can initialize SQLite and history.

Gaps:

1. **Required CLI options are missing.**  
   The spec requires JSON output, directory/language filtering, and logical-neighbor display. The CLI only supports `--limit` and `--commits`.

2. **Scoring does not follow `docs/Plan-Phase2.md`.**  
   Phase 19 specifies normalized factors using `value / max(all_values)`. The implementation uses `frequency / total_commits` and `complexity / 50.0`.

3. **Risk density formula conflicts across docs and is not documented in code.**  
   Track 25 says weighted sum; `Plan-Phase2.md` says normalized multiplication. The implementation uses an unconfigurable 50/50 sum.

4. **Sorting is not fully deterministic and can panic on NaN.**  
   Both `src/commands/hotspots.rs` and `src/impact/hotspots.rs` use `b.score.partial_cmp(&a.score).unwrap()` without a path tiebreaker.

5. **SQLite row errors are silently dropped.**  
   The hotspot query uses `.filter_map(|res| res.ok())`, which hides malformed rows or conversion failures.

6. **Duplicate implementation exists.**  
   `src/commands/hotspots.rs` duplicates logic from `src/impact/hotspots.rs` instead of calling the shared engine. This weakens SRP and increases drift risk.

7. **No dedicated hotspot test was found.**  
   The Phase 2 plan calls for scoring math and deterministic ranking tests.

## Track 26: Predictive Verification

**Status: fail/partial**

What is implemented:

- `src/verify/predict.rs` exists.
- `VerificationPlan` accepts predicted files.
- Predicted files can add rule-driven verification steps.
- `--no-predict` exists on the `verify` command.

Critical gaps:

1. **Structural prediction is not implemented.**  
   The track spec requires predicting files that import changed files. `Predictor::predict()` only uses `packet.temporal_couplings`.

2. **The implementation contains placeholder/thought-process comments.**  
   `src/verify/predict.rs` includes comments such as:
   - "Let's re-read the spec carefully."
   - "For now, let's implement what we CAN with the packet data."
   - "I'll check if there are any existing tests for this."

   This is not production-quality code and strongly indicates the track was not finished.

3. **Temporal analysis is not computed in `verify`.**  
   The spec requires wiring temporal analysis and structural impact into `verify` before plan construction. `commands/verify.rs` only reads the latest stored packet and passes it to `Predictor`.

4. **Graceful degradation is missing.**  
   If temporal data is unavailable, the spec requires deterministic warnings and structural-only fallback. The current path simply returns no predictions when the packet is absent or has no temporal couplings.

5. **No predictor tests were found.**  
   `verify::plan` has a test for consuming a synthetic `PredictedFile`, but `Predictor::predict()` itself has no tests for structural, temporal, deduplication, ordering, or degradation behavior.

6. **Predicted-step deduplication can erase traceability.**  
   Plan construction deduplicates by command string after sorting. If a predicted command duplicates a direct rule command, the predicted reason may disappear.

## Track 27: LSP-Lite ChangeGuard Daemon

**Status: fail**

What is implemented:

- `tower-lsp-server` is optional behind the `daemon` feature.
- `src/output/lsp.rs` maps impact packets to LSP diagnostics.
- `src/commands/daemon.rs` exists.
- `cargo check --all-features` passes.

Critical gaps:

1. **There is no `src/daemon/` directory.**  
   The plan requires `src/daemon/mod.rs`, `server.rs`, `handlers.rs`, `state.rs`, and `lifecycle.rs`.

2. **The daemon is not an LSP server.**  
   `execute_daemon()` just calls `execute_watch(1000, true)`. It does not implement `LanguageServer`, JSON-RPC request handling, `textDocument/codeLens`, `textDocument/hover`, initialization, or shutdown.

3. **No constrained Tokio runtime is configured.**  
   `Cargo.toml` does not declare `tokio` as an optional direct dependency, and `commands/daemon.rs` does not create a `worker_threads(2)` runtime.

4. **No daemon lifecycle management exists.**  
   There is no PID file creation, stale PID detection, graceful shutdown, or broken-stdin self-termination.

5. **No read-only SQLite daemon state layer exists.**  
   The required read-only WAL connection, `SQLITE_BUSY` retry/backoff, and `data_stale: true` behavior are absent.

6. **`output/lsp.rs` is diagnostics-only.**  
   The plan requires Diagnostic, CodeLens, and Hover mapping. Only diagnostics are implemented.

7. **Feature-enabled clippy/test paths fail.**  
   `cargo clippy --all-targets --all-features -- -D warnings` fails inside `src/output/lsp.rs` tests because `ChangedFile` is not imported.

8. **No daemon lifecycle tests exist.**  
   The required `tests/daemon_lifecycle.rs` was not found.

## Track 28: Federated Intelligence

**Status: partial**

What is implemented:

- `src/federated/mod.rs`, `schema.rs`, `scanner.rs`, `impact.rs`, and `storage.rs` exist.
- `changeguard federate export`, `scan`, and `status` are registered.
- `FederatedSchema` contains `schema_version`, repo name, and public interfaces.
- Scanner uses `symlink_metadata()` and skips symlinks.
- Migrations include `federated_links` and `federated_dependencies`.
- Basic sibling discovery and symlink tests exist.

Gaps:

1. **Cross-repo impact resolution is not implemented.**  
   `src/federated/impact.rs` contains comments saying "In a real implementation..." and currently only adds a generic "Cross-repo monitoring active" risk reason. It does not compare interfaces, dependency edges, or changed sibling schemas.

2. **Federated dependency storage is unused.**  
   `save_federated_dependencies()` and `get_dependencies_for_sibling()` exist, but scan/export/status do not populate dependency rows or use them for impact resolution.

3. **No automatic sibling detection in `scan` or `impact`.**  
   The user workflow says `scan` or `impact` in Repo B automatically detects sibling schemas. Current impact only checks already-stored links.

4. **Path confinement is incomplete.**  
   The scanner iterates the parent directory and skips symlinks, but it does not canonicalize and verify that discovered entries are exactly one level above the repo root as required.

5. **Schema validation is missing.**  
   `load_schema()` accepts any JSON that deserializes to `FederatedSchema`. There is no version validation, JSON schema validation, or rejection of unsupported schema versions.

6. **Malformed schemas are logged, not surfaced as user diagnostics.**  
   The plan allows skipping malformed siblings, but requires clear diagnostic warnings. The current code only emits `tracing::warn!`, which many CLI users will never see.

7. **No `catch_unwind` defense exists.**  
   `docs/Plan-Phase2.md` specifically calls for parsing malicious schema files inside a `catch_unwind` boundary as defense in depth.

8. **Sibling scan cap is missing.**  
   The plan requires a default cap of 20 siblings. No cap or config exists.

9. **Secret redaction is not explicit in schema export.**  
   Export currently emits public symbols from SQLite. That likely avoids values by construction, but the track explicitly requires export redaction. There is no redaction pass in `execute_federate_export()`.

10. **Production `unwrap_or("unknown")` weakens error visibility.**  
   `execute_federate_export()` silently names a repo `unknown` if repo name extraction fails, rather than surfacing an actionable diagnostic.

## Track 29: Advanced Narrative Reporting

**Status: partial**

What is implemented:

- `src/gemini/narrative.rs` exists.
- `GeminiMode::Narrative` exists.
- Ask can run in narrative mode through `--mode narrative`.
- Prompt sanitization exists via `gemini::sanitize`.
- Secret redaction patterns exist in `impact/redact.rs`.
- Gemini process execution has a timeout.

Gaps:

1. **The requested CLI shape is not implemented.**  
   The track asks for `changeguard ask --narrative` or a dedicated subcommand. The implementation adds `--mode narrative`, which may be acceptable, but it is not documented as the requested UX.

2. **Narrative prompt generation is conditional on query text.**  
   `commands/ask.rs` only uses `NarrativeEngine::generate_risk_prompt()` when mode is `Narrative` and the query is exactly `"summary"`. Other narrative queries do not use the narrative engine.

3. **Token budgeting is not wired into Gemini execution.**  
   `ImpactPacket::truncate_for_context()` exists, but `commands/ask.rs` and `gemini/wrapper.rs` never call it. The required 80% of 128k context-window estimator is absent from `wrapper.rs`.

4. **No truncation annotation is appended.**  
   The plan requires `"Packet truncated for Gemini submission"` when truncation occurs. No execution path currently adds that annotation.

5. **Gemini is not invoked in a real CLI mode.**  
   `run_query()` starts `gemini` and writes prompts to stdin. It does not pass an `analyze` mode argument or otherwise control Gemini mode beyond prompt wording.

6. **Gemini failure fallback is incomplete.**  
   The plan requires saving the raw/redacted impact packet as a fallback artifact when Gemini exits non-zero. `run_query()` prints stderr and returns an error but does not write a fallback artifact.

7. **Missing Gemini CLI error is less actionable than specified.**  
   The spec asks for `"Gemini CLI not found. Install Gemini CLI to enable narrative summaries."` Current spawn failure is `"Failed to spawn gemini: ..."` with no install guidance.

8. **No golden prompt tests were found.**  
   The spec requires byte-for-byte deterministic prompt tests for narrative generation.

9. **Impact packets are persisted before redaction.**  
   `commands/impact.rs` saves the packet to SQLite before `packet.finalize()` and before `redact_secrets()`. The disk report is redacted, but the SQLite packet used by `ask` may still contain unredacted data. This is a Phase 2 safety issue.

## Engineering Standards Findings

### 1. Verification gates fail

Severity: CRITICAL

Evidence:

- `cargo fmt --check` fails across multiple Phase 2 files.
- `cargo test` fails because `tests/cli_watch.rs` calls a stale function signature.
- `cargo clippy --all-targets --all-features -- -D warnings` fails.

This alone prevents accepting Phase 2 as complete.

### 2. Placeholder/incomplete implementation comments remain in production code

Severity: HIGH

Evidence:

- `src/verify/predict.rs` contains implementor reasoning comments.
- `src/federated/impact.rs` contains "In a real implementation..." comments and implements a generic warning instead of the planned behavior.

These are not acceptable in completed production paths.

### 3. Production `unwrap()` remains

Severity: HIGH

Examples:

- `src/commands/impact.rs`: `Utf8Path::from_path(relative_path).unwrap()`
- `src/commands/hotspots.rs`: `partial_cmp(...).unwrap()`
- `src/impact/hotspots.rs`: `partial_cmp(...).unwrap()`

Phase 2 explicitly bans `unwrap()` and `expect()` in production logic.

### 4. Silent data loss and error suppression remain

Severity: HIGH

Examples:

- Hotspot queries drop SQLite row errors with `.filter_map(|res| res.ok())`.
- Federated malformed schema errors are only logged with `tracing::warn!`.
- Federated impact silently ignores schemas that fail to parse.
- Verification DB persistence warnings are logged but not surfaced in the report.

### 5. Secret safety is incomplete

Severity: HIGH

Disk report redaction exists, but `commands/impact.rs` persists the packet to SQLite before redaction. `ask` reads from SQLite, meaning Gemini prompt construction can start from an unredacted packet. Sanitization later helps, but the local DB still stores the raw packet and any missed pattern goes directly into the prompt path.

### 6. Determinism is inconsistent

Severity: MEDIUM

Examples:

- Hotspot sorting does not have a path tiebreaker in command/shared engine paths.
- `partial_cmp().unwrap()` can panic if a score is NaN.
- Federation sorting only by repo name can produce unstable order when duplicate names exist.
- Temporal traversal uses breadth-first history rather than the specified first-parent order.

### 7. SRP drift remains

Severity: MEDIUM

Examples:

- `commands/hotspots.rs` duplicates hotspot calculation instead of using `impact/hotspots.rs`.
- `commands/daemon.rs` delegates to watch rather than owning daemon lifecycle/server setup.
- `federated/impact.rs` does schema loading and impact mutation together.

## Testing Gaps

Missing or inadequate Phase 2 tests:

- No real git-history fixture test for temporal coupling.
- No TypeScript complexity scoring test.
- No syntax-error/partial-AST complexity test.
- No unsupported-language complexity test.
- No large-file complexity cap test.
- No hotspot scoring/golden JSON test.
- No predictor unit tests.
- No CLI predictive verification test that proves actual prediction behavior.
- No daemon lifecycle test.
- No SQLite busy/backoff daemon test.
- No CodeLens/Hover mapping tests.
- No federated malformed-schema diagnostic test.
- No federated dependency impact-resolution test.
- No narrative golden prompt test.
- No token budget/truncation test wired to Gemini wrapper behavior.

## Priority Action Items

### Critical

1. Restore green verification gates: run `cargo fmt`, fix `tests/cli_watch.rs`, and fix clippy all-features failures.
2. Replace the Track 27 daemon stub with a real feature-gated LSP implementation or mark the track incomplete.
3. Rework `src/verify/predict.rs` to implement structural prediction, remove placeholder comments, add degradation diagnostics, and add predictor tests.
4. Move impact packet finalization/redaction before SQLite persistence so the DB does not store raw unredacted packets.

### High

5. Implement real federated impact resolution using stored dependency edges; remove the generic "monitoring active" placeholder.
6. Wire token budgeting/truncation into `gemini/wrapper.rs` or `commands/ask.rs`, including the required truncation annotation.
7. Remove production `unwrap()` paths from impact/hotspot code.
8. Add hotspot JSON output, path/language filtering, and deterministic score tie-breaking.
9. Add schema validation/version checks and canonical path confinement to federated scanning.
10. Surface malformed federated schema diagnostics to the user instead of logging only.

### Medium

11. Implement first-parent traversal for temporal history and add a real git fixture test.
12. Add complexity degradation fields for syntax errors, unsupported languages, and capped large files.
13. Add TypeScript and golden-value complexity tests.
14. Remove duplicate hotspot logic from `commands/hotspots.rs`.
15. Add narrative golden prompt and token-budget tests.
16. Add daemon lifecycle and SQLite contention tests if the daemon track remains in scope.

## Final Verdict

The repository is no longer missing the broad Phase 2 source files, but many Phase 2 behaviors are incomplete or shallow. The current state should be treated as **Phase 2 scaffolding plus partial implementation**, not completion.

The strongest blockers are objective: formatting, tests, and clippy do not pass. Beyond that, Track 27 is not implemented, Track 26 has visible unfinished logic, and Track 28's core impact behavior is placeholder-grade. Phase 2 should not be accepted until those are corrected and verified with targeted tests.

# Audit 5: Observability & Intelligence Expansion

Date: 2026-05-03

Scope: audited `docs/observability-plan.md`, `conductor/conductor.md`, `conductor/trackM1-1` through `conductor/trackM6-2`, the current source tree, tests, CI gates, and ChangeGuard command output.

## Verdict

Milestone M is structurally present and mostly integrated: the repo contains the expected `embed`, `docs`, `local_model`, `retrieval`, `observability`, and `contracts` subsystems; new config sections and SQLite tables exist; `impact`, `ask`, `verify`, `index`, and `doctor` have M-track wiring; and the standard Rust CI gate is green.

It is not complete against `docs/observability-plan.md`. The biggest gaps are in M5/M6 behavior and standards compliance:

1. Observability and contract risk reasons are added before risk analysis, then overwritten by `analyze_risk`, so normal impact runs can drop the very signals these tracks were supposed to surface.
2. M5 observability does not implement the planned per-service Prometheus model, service mapping, GET query shape, log chunk embedding, diff/log semantic similarity, or packet shape.
3. M6 contract matching embeds changed file contents during `impact` instead of using pre-indexed `entity_type = 'file'` embeddings as planned, violating the no-embedding-on-hot-path principle.
4. `cargo deny check` fails due dependency policy issues.
5. The full M-track implementation is still uncommitted in this worktree, so the required red/green commit trace is not demonstrable.

## Verification Performed

Passing:

* `cargo fmt --all -- --check`
* `cargo clippy --all-targets --all-features -- -D warnings`
* `cargo test --workspace`: 591 library/unit tests plus integration tests passed
* Rebuilt and reinstalled release binary:
  * `cargo build --release`
  * copied `target/release/changeguard.exe` to `C:\Users\RyanB\.cargo\bin\changeguard.exe`
* `changeguard doctor`: passed, local model reported `Not configured`
* `changeguard impact --summary`: completed; reported `HIGH risk | 39 changed | 199 couplings | 18 partial`
* `changeguard hotspots`: completed and reported current hotspots

Failing:

* `cargo deny check`: fails. The direct failures are rejected `CDLA-Permissive-2.0` license entries through `webpki-roots` and an unmaintained `async-std` advisory through dev dependency `httpmock`.

## Findings

### High: M5/M6 Risk Reasons Are Overwritten

`execute_impact` enriches docs, observability, and contracts before calling `analyze_risk` (`src/commands/impact.rs:211`, `src/commands/impact.rs:217`, `src/commands/impact.rs:227`, `src/commands/impact.rs:234`). M5 and M6 append risk reasons during enrichment (`src/observability/mod.rs:119`, `src/observability/mod.rs:124`, `src/commands/impact.rs:1824`, `src/commands/impact.rs:1828`).

`analyze_risk` then replaces the whole reason list (`src/impact/analysis.rs:518`). That means observability threshold reasons and public contract reasons are not reliably present in final packets when normal rule-based risk analysis runs. This misses core M5/M6 acceptance criteria.

### High: Observability Implementation Does Not Match M5 Plan

The plan requires service-based Prometheus queries using `observability.service_map`, one error-rate query and one latency query per matched service, through the Prometheus HTTP API GET endpoint. The implementation instead runs two generic queries with no service label interpolation (`src/observability/mod.rs:50`, `src/observability/mod.rs:51`) and sends POST form requests (`src/observability/prometheus.rs:22`).

The plan requires error-rate threshold breaches to elevate risk by one tier and add a specific reason. The implementation maps signals to `Critical`/`Warning` and adds a generic reason before risk analysis (`src/observability/mod.rs:74`, `src/observability/mod.rs:119`), which is then vulnerable to being overwritten.

The planned packet shape was a compact `Option<ObservabilitySignal>` with service summaries, anomaly count, and risk elevation. The actual packet uses `Vec<ObservabilitySignal>` (`src/impact/packet.rs:447`) where each signal can contain an excerpt (`src/observability/signal.rs:20`-`src/observability/signal.rs:25`). This is a material design deviation.

### High: Log Scanning Skips Semantic Similarity

The plan requires 20-line log chunks, local embedding, cosine similarity against the current diff embedding, anomaly threshold `> 0.6`, and raw log content excluded from the packet. The implementation is keyword-based only: it scans for `ERROR`, `FATAL`, `panic`, and `exception` (`src/observability/log_scanner.rs:26`-`src/observability/log_scanner.rs:32`), reads whole log files (`src/observability/log_scanner.rs:50`), groups matching lines, and stores sanitized excerpts in packet signals (`src/observability/log_scanner.rs:114`).

The fallback keyword behavior is acceptable only when embeddings are unavailable. Here it is the primary implementation, so M5 is only partially implemented.

### High: Contract Matching Embeds During Impact

The plan says `execute_impact()` should retrieve changed file embeddings from SQLite where `entity_type = 'file'`, skip files with missing embeddings, and compare those vectors to endpoint embeddings. The implementation reads changed files from disk (`src/contracts/matcher.rs:48`) and calls `embed_long_text()` during matching (`src/contracts/matcher.rs:63`).

This violates the plan's "No embedding on hot path" principle and changes degradation behavior. It also means contract matching depends on live local model availability during `impact`, not just during indexing.

### Medium: Document Chunk Overlap Is Not Implemented

The plan requires deterministic heading-boundary splitting with a 512-token cap and 64-token overlap. `chunk_markdown` accepts `overlap_tokens` but explicitly ignores it (`src/docs/chunker.rs:16`, `src/docs/chunker.rs:21`). It also drops sections below 50 estimated tokens (`src/docs/chunker.rs:28`), which was not specified and can skip short ADR decisions or README sections.

### Medium: Dependency Policy Gate Is Not Green

`cargo deny check` fails. This conflicts with `docs/observability-plan.md` section 11, which says new dependency additions must pass `cargo audit` and `cargo deny check`.

The failures are actionable:

* Add or review policy for `CDLA-Permissive-2.0` due `webpki-roots`.
* Replace, upgrade, or explicitly allow the `httpmock` chain that pulls `async-std`, or change the mock-server strategy.
* Add a crate license field for `changeguard`.

### Medium: TDD/Commit Lifecycle Was Not Preserved in the Worktree

`git status --short` shows the M-track implementation is uncommitted: many modified source/test files, untracked `src/contracts`, `src/observability`, `src/retrieval`, untracked conductor track directories, and untracked `docs/observability-plan.md`.

Because there are no committed red/green boundaries for M1-1 through M6-2 in the current worktree, the repository standard "red commit then green commit(s)" cannot be verified.

### Medium: Tests Are Broad but Not All Planned Verification Gates Exist as Named Integration Tests

The implementation has many inline unit tests and the workspace test suite is green. However, the plan explicitly listed integration-style test files such as `tests/embed_storage.rs`, `tests/doc_chunking.rs`, `tests/local_model_context.rs`, `tests/semantic_test_prediction.rs`, `tests/observability_signal.rs`, and `tests/contract_matching.rs`. Those files do not exist.

Equivalent coverage exists in module tests for many areas, but the missing integration tests leave gaps for CLI-level and packet-level acceptance behavior, especially M5/M6 enrichment.

### Low: Placeholders/TODOs

No new production `TODO`, `FIXME`, `todo!`, `unimplemented!`, or stub markers were found in the M source modules. Existing historical TODO/placeholder text remains in older docs and conductor files. One M implementation comment is effectively a deferred feature: `overlap_tokens` is "reserved for future use and currently ignored" in `src/docs/chunker.rs:16`.

## Track Coverage

M1-1 and M1-2: Mostly complete. Config, migrations, embedding client/storage, similarity, budget helpers, and doctor local-model status exist. Tests cover storage, similarity, batch embedding, disabled-local-model degradation, and budget behavior.

M2-1 and M2-2: Mostly complete with one chunking deviation. Crawler, chunker, doc index, retrieval, reranking hooks, `relevant_decisions`, impact enrichment, and ask formatting exist. Missing overlap behavior means the exact chunking requirement is not fully met.

M3-1 and M3-2: Mostly complete. Local OpenAI-compatible completion client, context assembly, backend enum/routing, `--backend local/gemini`, auto-selection, and config reporting exist. CI passes with local model disabled.

M4-1 and M4-2: Mostly complete. Diff outcome recording, semantic score computation, score blending, `semantic_weight`, and `--explain` wiring exist. The implementation degrades when local embeddings are disabled.

M5-1 and M5-2: Partial. Files and basic functionality exist, but core plan semantics are missing: per-service Prometheus mapping, exact PromQL queries, GET API shape, log embedding/similarity, compact packet schema, and durable final risk elevation.

M6-1 and M6-2: Partial. Parser and indexing are largely present, including OpenAPI/Swagger support and stale cleanup. Matching and enrichment do not meet the planned indexed-file-embedding flow, and final risk reasons can be overwritten.

## Required Remediation

1. Move observability and contract risk integration into `analyze_risk`, or run those enrichments after `analyze_risk` and explicitly elevate `packet.risk_level` without later overwrite.
2. Rework M5 around `service_map`: derive services from changed files, issue per-service Prometheus API requests, compute `ServiceSignal`s, store planned snapshot columns or document the schema change, and only persist compact summaries in packets.
3. Implement log chunk embedding and diff/log cosine similarity; keep keyword matching strictly as fallback when embeddings are unavailable.
4. Change M6 matching to use indexed `entity_type = 'file'` embeddings and skip missing file embeddings during impact.
5. Implement chunk overlap or update the plan/conductor with an explicit accepted deferral.
6. Fix `cargo deny check`.
7. Commit the M-track work in reviewable units, or at minimum document why the required red/green commit lifecycle could not be reconstructed.


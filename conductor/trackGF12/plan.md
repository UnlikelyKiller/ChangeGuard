# Track GF12 Plan: Local Model Client Split

## Phase 0: Baseline and Guardrails

- [ ] Confirm ledger state: `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF12 --category REFACTOR --message "Local model client split by endpoint provider"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `cargo test local_model` and record the baseline test count.
- [ ] Run `cargo check --all-targets --all-features` and confirm clean.
- [ ] Confirm the public API list from the spec (callers of `complete`, `gemini_complete`, `ping_completions`, `has_ollama_cloud_fallback`, `ChatMessage`, `CompletionOptions`) still matches via grep.
- [ ] Read `complete_with_endpoint` in full; decide between keeping it whole in `client.rs` (default) or splitting protocol branches into `ollama.rs`/`openai.rs` helpers.

Definition of done: Public API confirmed; split strategy for `complete_with_endpoint` decided; ledger open.

## Phase 1: Types and Utilities Extraction

- [ ] Create `src/local_model/client/` directory. `client.rs` stays as the facade — do NOT rename to `client/mod.rs`, and do NOT add module declarations to `src/local_model/mod.rs` (the new modules are children of `client`, declared inside `client.rs`).
- [ ] Create `client/types.rs`; move `EndpointKind`, `EndpointTarget`, `CompletionEndpoint` (and optionally `ChatMessage`, `CompletionOptions` with `pub use types::{ChatMessage, CompletionOptions};` in the facade so external imports keep compiling).
  - Add `mod types;` to `client.rs`; rewire internal uses.
  - Run `cargo check --all-targets --all-features`.
- [ ] Create `client/util.rs`; move `detect_endpoint_kind`, `completion_target`, `check_base_url_warnings`, `transport_is_timeout` (mark `pub(super)`).
  - Add `mod util;` to `client.rs`; rewire callers.
  - Run `cargo check`.

Definition of done: Shared types and utilities extracted; external imports of `ChatMessage`/`CompletionOptions` still compile; clean check.

## Phase 2: Provider Extractions

Move one provider at a time. After each: `cargo check --all-targets --all-features` and `cargo test local_model`.

- [ ] Create `client/gemini.rs`; move `gemini_complete` and all Gemini-specific types and private helpers.
  - Add `mod gemini; pub use gemini::gemini_complete;` to `client.rs`.
  - Run `cargo check` + targeted tests.
- [ ] Create `client/cloud.rs`; move `has_ollama_cloud_fallback`, `ollama_cloud_endpoint`.
  - Add `mod cloud; pub use cloud::has_ollama_cloud_fallback;` to `client.rs` (`ollama_cloud_endpoint` stays `pub(super)` — it has no external callers).
  - Run `cargo check` + targeted tests.
- [ ] Per the Phase 0 decision, either:
  - Keep `complete_with_endpoint` whole in `client.rs`, extracting only protocol response types: move `OllamaChatResponse`, `OllamaChatMessage`, `ollama_native_num_predict` to `client/ollama.rs` and `CompletionResponse`, `Choice`, `ChoiceMessage` to `client/openai.rs` (both `pub(super)`); **or**
  - Split the request-build/response-parse branches into `pub(super)` functions in `ollama.rs`/`openai.rs` with `complete_with_endpoint` reduced to routing.
  - Run `cargo check` + targeted tests after each move.
- [ ] Verify `client.rs` now contains only: `ping_completions`, `complete`, `complete_with_endpoint` (or its routing remnant), `mod` declarations, and `pub use` re-exports.

Definition of done: Gemini and cloud fully extracted; Ollama/OpenAI extracted to the depth the Phase 0 decision supports; facade holds only public API and dispatch.

## Phase 3: Test Relocation and Cleanup

- [ ] Relocate `#[cfg(test)]` tests to the module whose code they exercise; keep endpoint-routing and `complete` end-to-end tests in the facade.
- [ ] Run `cargo test local_model` and confirm count is at or above the Phase 0 baseline.
- [ ] Remove unused imports; run `cargo clippy --all-targets --all-features -- -D warnings`.

Definition of done: No lint warnings; all tests pass at baseline or above.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit: `changeguard ledger commit <tx-id> --summary "Completed Track GF12: local model client split by endpoint provider" --reason "1,170-line file with 4 distinct protocol paths split into focused child modules"`. If the git pre-commit hook removed the sidecar and `ledger status` still shows 1 pending after the git commit, run `ledger commit` again immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.
- [ ] Mark all tasks `- [x]` in this plan and set Status: Completed in `conductor/conductor.md`.

Definition of done: Full gates pass; installed binary matches source; ledger clean; conductor registry current.

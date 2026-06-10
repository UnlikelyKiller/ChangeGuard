# Track GF12: Local Model Client Split

## Objective

Split `src/local_model/client.rs` (1,170 lines, ~583 production) by endpoint provider. The file currently implements four distinct completion protocols — Ollama native `/api/chat`, OpenAI-compatible `/v1/chat/completions`, Gemini REST API, and Ollama Cloud fallback routing — alongside shared types and detection utilities. Each protocol is a separate I/O concern with different request/response shapes.

## Evidence

- 1,170 lines total; `#[cfg(test)]` begins at line 584, so ~583 production lines
- Four distinct endpoint protocols identified:
  - Ollama native `POST /api/chat` — custom request/response types (`OllamaChatResponse`, `OllamaChatMessage`), `num_predict` mapping, and `thinking` field handling
  - OpenAI-compatible `POST /v1/chat/completions` — standard `CompletionResponse`/`Choice`/`ChoiceMessage` deserialization
  - Gemini REST API — `gemini_complete` with its own JSON shape, API key injection, and model-name routing
  - Ollama Cloud fallback — `has_ollama_cloud_fallback`, `ollama_cloud_endpoint` — detection and routing to a hosted Ollama instance via `OLLAMA_CLOUD_*` config
- Shared utilities: `EndpointKind`, `EndpointTarget`, `CompletionEndpoint`, `detect_endpoint_kind`, `completion_target`, `check_base_url_warnings`, `transport_is_timeout`, `ollama_native_num_predict`
- **Public API surface (verified 2026-06-10 via caller grep)** — all of these are imported externally as `crate::local_model::client::*` and must remain at that path:
  - `complete` — `ai/intent_drafter.rs`, `ai/semantic_extractor.rs`, `commands/ask.rs`, `verify/explanation.rs`
  - `gemini_complete` — `ai/semantic_extractor.rs`
  - `ping_completions` — `commands/ask.rs`, `commands/doctor.rs`
  - `has_ollama_cloud_fallback` — `commands/ask.rs`, `commands/config_verify.rs`
  - `ChatMessage` — `ai/*`, `local_model/context.rs`, `verify/explanation.rs`
  - `CompletionOptions` — `ai/*`, `commands/ask.rs`, `verify/explanation.rs`
- Note: this is a secondary/borderline candidate — 583 production lines is smaller than the primaries, and all code is functionally related. The split is justified by the four protocol shapes having genuinely different I/O contracts.

## Scope

Facade pattern: keep `src/local_model/client.rs` as the facade and add a sibling `src/local_model/client/` directory — the GF4 shape (`db.rs` facade + `db/` submodules). `mod gemini;` declared inside `client.rs` resolves to `client/gemini.rs`. Do NOT declare new modules in `src/local_model/mod.rs`; they are children of `client`, not siblings.

| Module | Assigned items |
|---|---|
| `client.rs` (facade) | `ChatMessage`, `CompletionOptions` (or re-exported from `client/types.rs`), `ping_completions`, `complete`, `complete_with_endpoint` (if kept whole — see notes), `mod` declarations, `pub use` re-exports of `gemini_complete` and `has_ollama_cloud_fallback` |
| `client/types.rs` | Shared internal types: `EndpointKind`, `EndpointTarget`, `CompletionEndpoint`; optionally `ChatMessage`/`CompletionOptions` with facade re-export |
| `client/ollama.rs` | `OllamaChatResponse`, `OllamaChatMessage`, `ollama_native_num_predict`, and the Ollama-native request/response handling |
| `client/openai.rs` | `CompletionResponse`, `Choice`, `ChoiceMessage`, and the OpenAI-compatible request/response handling |
| `client/gemini.rs` | `gemini_complete` + all private Gemini helpers and response types |
| `client/cloud.rs` | `has_ollama_cloud_fallback`, `ollama_cloud_endpoint` |
| `client/util.rs` | `detect_endpoint_kind`, `completion_target`, `check_base_url_warnings`, `transport_is_timeout` |

## Non-Goals

- No behavior changes to any completion path.
- No changes to public function signatures.
- No new endpoint support.
- No changes to `LocalModelConfig` or `GeminiConfig` (they live in `src/config/model.rs`).
- No edits to `src/local_model/mod.rs` beyond what already exists (`pub mod client;` is sufficient).
- No touching `.changeguard` state files.

## Implementation Notes

- `complete_with_endpoint` (~147 lines) dispatches across `EndpointKind` variants and touches both Ollama and OpenAI response types. Default decision: keep it whole in `client.rs` and have it call `pub(super)` helper functions in `ollama.rs`/`openai.rs` for the protocol-specific request building and response parsing. Only split the function itself if that extraction turns out clean.
- Acceptable minimal outcome: if protocol dispatch proves tightly entangled, the track still succeeds with `types.rs`, `util.rs`, `gemini.rs`, and `cloud.rs` extracted and the Ollama/OpenAI paths remaining in `client.rs` — that alone removes ~250 production lines from the facade. Do not force the ollama/openai split if it requires contorting the dispatch.
- Response types (`CompletionResponse`, `OllamaChatResponse`, etc.) are deserialization-only and not imported externally — they can be private to their protocol module.
- The `#[cfg(test)]` block (line 584+, ~587 lines, likely httpmock-based) is large; relocate tests to the module whose code they exercise, keeping endpoint-routing tests in the facade.

## Verification Strategy

Targeted (run after each module move):
- `cargo check --all-targets --all-features`
- `cargo test local_model`

Final:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/local_model/client.rs` contains the public API surface, dispatch, `mod` declarations, and re-exports — protocol-specific types and Gemini/cloud logic are in child modules.
- All six public symbols (`complete`, `gemini_complete`, `ping_completions`, `has_ollama_cloud_fallback`, `ChatMessage`, `CompletionOptions`) remain importable from `crate::local_model::client`.
- All existing tests pass at or above the baseline count.
- Full verification and reinstall pass.
- Ledger transaction committed; `changeguard ledger status --compact` shows `0 pending, 0 unaudited drift`.

## Risks

- `complete_with_endpoint` entanglement: if splitting it across modules introduces borrow-checker friction or duplicated dispatch, fall back to the minimal outcome described in the notes rather than forcing the full table.
- The httpmock test suite may pin exact request shapes; any accidental change to request building will surface there — treat test failures as behavior regressions, not test brittleness.

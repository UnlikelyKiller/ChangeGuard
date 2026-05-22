# Track K12 Plan: Local Model Timeout and Readiness UX

## Phase 1: Baseline Tests
- [ ] Add tests for unreachable local completion endpoint behavior.
- [ ] Add tests for separate embedding and completion readiness reporting.
- [ ] Add tests for split endpoint config: embedding requests use `embedding_base_url`, completions use `generation_base_url`.
- [ ] Add backward-compatibility tests for legacy `base_url` only configs.
- [ ] Capture current timeout defaults and config behavior.

## Phase 2: Probe Design
- [ ] Add optional `embedding_base_url` and `generation_base_url` fields with `base_url` fallback.
- [ ] Introduce a short probe timeout distinct from generation timeout.
- [ ] Apply probe timeout in `doctor` and preflight checks.
- [ ] Route embedding clients and semantic indexing to the embedding endpoint.
- [ ] Route completion clients and `ask --backend local` to the generation endpoint.
- [ ] Keep full request timeout for actual completions.

## Phase 3: UX and Verification
- [ ] Improve local-model failure text with endpoint, timeout, and suggested next step.
- [ ] Verify LLM2 config: Gemma chat on `127.0.0.1:8081`, Nomic Embed v1.5 on `127.0.0.1:8083`.
- [ ] Verify legacy single-base-url config still works with a combined OpenAI-compatible server.
- [ ] Run `doctor` with the local model offline.
- [ ] Run `ask --backend local` with the local model offline.
- [ ] Run `ask --backend local` with LLM2 online.
- [ ] Run semantic indexing/search with only the embedding endpoint online.
- [ ] Run `cargo install --path . --force` and repeat installed-binary local-model checks.
- [ ] Run full CI gate.

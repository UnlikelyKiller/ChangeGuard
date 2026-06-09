## Plan: Track M3-1 — Local Model Client & Context Assembly

### Phase 1: Completions Client
- [ ] Task 1.1: Create `src/local_model/client.rs` with `complete(base_url: &str, model: &str, system_prompt: &str, user_prompt: &str, timeout_secs: u64) -> Result<String>`.
- [ ] Task 1.2: POST to `{base_url}/v1/chat/completions` with `{"model": model, "messages": [{"role":"system","content":"..."},{"role":"user","content":"..."}], "stream": false}`.
- [ ] Task 1.3: Parse `choices[0].message.content` as the response string.
- [ ] Task 1.4: On 503, retry once after 2s; on any other error, return `Err` immediately.
- [ ] Task 1.5: When server is unreachable, return `Err` with message: "Local model server not reachable at {base_url}. Start llama-server or use --backend gemini."
- [ ] Task 1.6: Write unit test: mock server returns valid completion → function returns content string.
- [ ] Task 1.7: Write unit test: mock server returns 503 → retries once → succeeds on second attempt.
- [ ] Task 1.8: Write unit test: server unreachable → returns `Err` with expected message.

### Phase 2: Shared Rerank Client
- [ ] Task 2.1: Create `src/local_model/rerank.rs` as a thin re-export or alias pointing to `src/retrieval/rerank.rs`. Both tracks use the same implementation; do not duplicate code.
- [ ] Task 2.2: Confirm `src/retrieval/rerank.rs` is `pub` and accessible from `src/local_model/`.

### Phase 3: Context Assembly
- [ ] Task 3.1: Create `src/local_model/context.rs` with `assemble_context(config: &LocalModelConfig, packet: &ImpactPacket, mode: GeminiMode, query: &str, diff: Option<&str>) -> String`.
- [ ] Task 3.2: Component order (descending priority for budget):
  1. User query (never truncated)
  2. Impact packet summary: risk_level, risk_reasons, top-3 changed files (~500 token target)
  3. Retrieved `relevant_decisions` from packet (formatted block)
  4. Top-5 temporal couplings summary
  5. Top-5 hotspots summary
  6. Full diff (only for `ReviewPatch` mode, if available)
- [ ] Task 3.3: Apply `enforce_budget` to components 2–6 together against `config.context_window - 500` (reserve 500 tokens for generation).
- [ ] Task 3.4: Log `WARN` with component name when any component is trimmed.
- [ ] Task 3.5: Write unit test: context with all components under budget → all components present in output.
- [ ] Task 3.6: Write unit test: context overflow → components trimmed from lowest priority (diff) first; query always present.
- [ ] Task 3.7: Write unit test: `ReviewPatch` mode with diff → diff is included at end.
- [ ] Task 3.8: Write unit test: non-`ReviewPatch` mode → diff is not included.

### Phase 4: Module Declaration
- [ ] Task 4.1: Create `src/local_model/mod.rs` exporting `client`, `context`, `rerank` submodules.
- [ ] Task 4.2: Add `pub mod local_model;` to `src/lib.rs`.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib local_model` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.

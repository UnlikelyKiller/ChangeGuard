# Track K2: Intelligence Precision Plan

## Phase 1: Context Assembly Refactor
- [ ] Inspect `src/commands/ask.rs` and `src/local_model/context.rs`.
- [ ] Implement `is_clean_state` check (no active tx, no uncommitted changes).
- [ ] Create a `CodebaseInquiryPrompt` variant in `src/gemini/prompt.rs`.
- [ ] Adjust `assemble_context` to allocate more tokens to `code_snippets` in clean state.

## Phase 2: Hallucination Mitigation
- [ ] Filter out "Impact Report" sections from the prompt if the report is essentially empty.
- [ ] Ensure `retrieval::query` is called with high enough `top_k` to cover the topic.
- [ ] Add explicit instruction: "If no changes are provided, answer based on the retrieved code snippets only."

## Phase 3: Verification
- [ ] Manual check: `changeguard ask --semantic "List all sub-tracks in Milestone J"` (from clean state).
- [ ] Add unit test for context prioritizer.
- [ ] CI Gate.

# Track K2: Intelligence Precision Plan

## Phase 1: Adaptive Context Assembly
- [ ] Implement `is_impact_empty(packet)` utility.
- [ ] Refactor `assemble_context` to accept an `AdaptiveMode` enum (ChangesFocus, CodebaseFocus).
- [ ] In `CodebaseFocus` mode:
    - [ ] Allocate 90% of budget to `relevant_chunks`.
    - [ ] Add explicit system prompt: "Answer from retrieval; cite [path:line]."
- [ ] Update `execute_ask` to detect clean state and pivot mode.

## Phase 2: Retrieval Hardening
- [ ] Implement `QueryRefiner`: use a lightweight regex to extract likely keywords/symbols from query.
- [ ] Boost chunks that contain extracted keywords in semantic retrieval results.
- [ ] Adjust `top_k` dynamically to fill the available context window (O(N) fill).

## Phase 3: Final Verification
- [ ] Manual check: `changeguard ask --semantic "How are migrations handled?"` in clean state.
- [ ] Verify citation format in model response.
- [ ] Add regression test for budget allocation logic.
- [ ] CI Gate.

## Plan: Track B7 - Verification Feedback Loop
### Phase 1: Verification Hook Implementation
- [ ] Task 1.1: Create `src/bridge/notify.rs` exposing a `push_verify_outcome(outcome)` function.
- [ ] Task 1.2: Update `src/verify/mod.rs` to call `push_verify_outcome` immediately upon processing the local verification results.
- [ ] Task 1.3: Enforce fire-and-forget behavior, ensuring IPC bottlenecks or disconnects do not delay `verify` completion.
- [ ] Task 1.4: Write a unit test isolating `verify` to confirm that mocked notification failures never cascade into command failures.

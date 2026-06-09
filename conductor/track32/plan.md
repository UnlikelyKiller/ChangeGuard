## Plan: Predictive Verification Completion
### Phase 1: Code Cleanup and Refactoring
- [ ] Task 1.1: Audit `src/verify/predict.rs` and remove all placeholder comments, thought-process notes, and non-production logic.
- [ ] Task 1.2: Refactor `commands/verify.rs` to compute or retrieve temporal analysis robustly before passing data to `Predictor::predict()`.

### Phase 2: Implement Core Prediction Engines
- [ ] Task 2.1: Implement structural prediction in `Predictor::predict()` to identify files importing or relying on changed files.
- [ ] Task 2.2: Ensure `Predictor::predict()` correctly merges structural predictions with `packet.temporal_couplings`.
- [ ] Task 2.3: Implement graceful degradation; if temporal analysis is missing or unavailable, emit deterministic warnings and fallback to structural-only mode.

### Phase 3: Traceability and Trace Deduplication
- [ ] Task 3.1: Update step deduplication in plan construction (`src/verify/plan.rs` or relevant deduplication logic) to properly merge reasons.
- [ ] Task 3.2: Ensure that if a predicted command matches a direct rule command, the predicted reason is retained, preserving traceability.

### Phase 4: Testing and Validation
- [ ] Task 4.1: Add unit tests to `Predictor::predict()` for structural and temporal prediction paths, deterministic ordering, and degradation behavior.
- [ ] Task 4.2: Add unit tests for deduplication retaining traceability.
- [ ] Task 4.3: Create CLI integration tests in `tests/` to prove real end-to-end predictive behavior.
- [ ] Task 4.4: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features` to ensure strict quality gates pass.

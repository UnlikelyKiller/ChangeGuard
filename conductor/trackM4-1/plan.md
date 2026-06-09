## Plan: Track M4-1 — Test Outcome Recording & Diff Embedding

### Phase 1: Diff Text Construction
- [ ] Task 1.1: Add `build_diff_text(packet: &ImpactPacket) -> String` in `src/verify/semantic_predictor.rs` (create file).
- [ ] Task 1.2: Concatenate changed file paths + changed symbol names from packet, space-separated. Use only the first 200 items to bound length.
- [ ] Task 1.3: Write unit test: packet with 3 changed files and 5 symbols → diff text contains all paths and symbol names.
- [ ] Task 1.4: Write unit test: packet with 0 changes → returns empty string.

### Phase 2: Outcome Recording
- [ ] Task 2.1: Add `record_test_outcomes(config: &Config, conn: &Connection, packet: &ImpactPacket, outcomes: &[(test_file: String, outcome: TestOutcome)]) -> Result<()>` to `src/verify/semantic_predictor.rs`.
- [ ] Task 2.2: Build diff text from packet; call `embed_and_store` with `entity_type = "diff"`, `entity_id = blake3(diff_text)`.
- [ ] Task 2.3: If embedding succeeds (returns `Ok(true)` or `Ok(false)`), get the embedding `id` from the `embeddings` table.
- [ ] Task 2.4: INSERT a row into `test_outcome_history` for each `(test_file, outcome)` pair, referencing the `diff_embedding_id`.
- [ ] Task 2.5: If `base_url` is empty, skip entirely without error.
- [ ] Task 2.6: Write unit test: `record_test_outcomes` with 3 outcomes inserts 3 rows in `test_outcome_history`.
- [ ] Task 2.7: Write unit test: recording same diff twice → embedding is reused (same `diff_embedding_id`), outcomes accumulate.
- [ ] Task 2.8: Write unit test: `base_url` empty → no rows inserted, no error.

### Phase 3: Hook into `execute_verify()`
- [ ] Task 3.1: In `src/commands/verify.rs`, after the verification runner completes, call `record_test_outcomes` with each test file's pass/fail outcome.
- [ ] Task 3.2: Infer outcome: exit code 0 = `pass`, non-zero = `fail`. Track the test file name from the `VerificationPlan` step.
- [ ] Task 3.3: Wrap the call in a non-blocking way: failure to record must never fail the verify command.
- [ ] Task 3.4: Write integration test: run `execute_verify` with a mock command; assert `test_outcome_history` has one new row.
- [ ] Task 3.5: Write test: outcome recording failure (mock DB error) does not propagate as an error from `execute_verify`.

### Phase 4: `TestOutcome` Type
- [ ] Task 4.1: Define `TestOutcome` enum in `src/verify/semantic_predictor.rs`: `Pass`, `Fail`, `Skip`.
- [ ] Task 4.2: Implement `as_str(&self) -> &'static str` returning `"pass"`, `"fail"`, `"skip"` for DB storage.
- [ ] Task 4.3: Write unit test: each variant serializes to the correct string.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib verify::semantic_predictor` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.

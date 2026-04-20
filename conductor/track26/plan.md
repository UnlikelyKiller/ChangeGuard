# Implementation Plan: Track 26 (Predictive Verification)

### Phase 1: Prediction Engine Core
- [ ] Task 1.1: Create `src/verify/predict.rs`. Define the `PredictedFile` struct containing `path: PathBuf` and `reason: PredictionReason` (enum: `Structural` or `Temporal`).
- [ ] Task 1.2: Implement the `Predictor` engine with a deterministic, depth-1 graph traversal mechanism.
- [ ] Task 1.3: Implement structural predictions by parsing the `ImpactPacket` imports to identify files dependent on the changed paths.
- [ ] Task 1.4: Implement temporal predictions by mapping changed paths against the `TemporalResult` coupling map.
- [ ] Task 1.5: Write unit tests in `predict.rs` to verify deterministic output (stable sorting and deduplication of predicted files).

### Phase 2: Verification Plan Expansion
- [ ] Task 2.1: Update `src/verify/plan.rs` to accept an optional `predicted_files: &[PredictedFile]` slice alongside the `ImpactPacket`.
- [ ] Task 2.2: Implement logic in `build_plan` to evaluate `rules.overrides` against predicted file paths.
- [ ] Task 2.3: Modify the `VerificationStep` creation to format descriptions for predicted steps (e.g., `"From rules (predicted impact on src/db.rs): cargo clippy"`).
- [ ] Task 2.4: Write unit tests in `plan.rs` to prove that predictive rules are applied correctly and do not duplicate commands already required by direct changes.

### Phase 3: CLI Integration and Degradation
- [ ] Task 3.1: Modify `src/commands/verify.rs` to compute or load temporal coupling data before executing the plan builder.
- [ ] Task 3.2: Wire `predict.rs` into the verification flow, merging structural and temporal signals.
- [ ] Task 3.3: Add a `--no-predict` flag to the `Verify` command in `src/cli.rs` and thread it through the command handler to bypass prediction logic when requested.
- [ ] Task 3.4: Implement graceful degradation for the temporal signal: if temporal data is missing or errors out, log an explicit warning (`"Insufficient history for temporal prediction..."`) and fall back to structural-only prediction without halting the command.

### Phase 4: Verification and Final Polish
- [ ] Task 4.1: Write a black-box integration test in `tests/cli_verify.rs` invoking `changeguard verify` with predictive conditions.
- [ ] Task 4.2: Verify that `cargo clippy` and `cargo test` pass cleanly.
- [ ] Task 4.3: Ensure no new dependencies were added to `Cargo.toml` unless strictly necessary (following YAGNI, standard library `HashMap` should suffice).
- [ ] Task 4.4: Mark all tasks in this plan as completed and submit the track.

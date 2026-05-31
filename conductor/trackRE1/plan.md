# Plan: Track RE1 (Decompose `src/commands/verify.rs`)

- [ ] 1. Create `src/verify/engine.rs`, `src/verify/predictor.rs`, and `src/output/verification.rs`.
- [ ] 2. Define the `VerificationContext` and `VerificationPlan` structures.
- [ ] 3. Move the prediction logic (historical/semantic blending) to `predictor.rs`.
- [ ] 4. Move the command execution and retry logic to `engine.rs`.
- [ ] 5. Move the terminal rendering and table generation to `src/output/verification.rs`.
- [ ] 6. Refactor `src/commands/verify.rs` to initialize these components and orchestrate the flow.
- [ ] 7. Verify with existing integration tests (`tests/cli_verify.rs`).

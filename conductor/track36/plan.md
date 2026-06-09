# Plan: Track 36 - Critical Remediation & Green CI

### Phase 1: Formatting and Linting
- [ ] Task 1.1: Run `cargo fmt` across the entire workspace to resolve formatting diffs in `cli.rs`, `commands/*.rs`, `daemon/*`, and tests.
- [ ] Task 1.2: Run `cargo clippy --all-targets --all-features --fix --allow-dirty` (or manually fix) the collapsible nested `if`s in `ask.rs`, `hotspots.rs`, `state.rs`, and the `and_then` issue in `temporal.rs`.
- [ ] Task 1.3: Verify `cargo clippy --all-targets --all-features -- -D warnings` passes locally without any warnings.

### Phase 2: Hotspot Error Handling
- [ ] Task 2.1: Locate the SQLite row processing loop in `src/impact/hotspots.rs` (currently using `.filter_map(|res| res.ok())`).
- [ ] Task 2.2: Refactor the loop to collect `Result`s instead of silently dropping errors. Accumulate errors into an error diagnostic list or fail the function with an actionable error.
- [ ] Task 2.3: Ensure the hotspot command output surfaces this failure rather than silently presenting a partial hotspot list as complete, ensuring compliance with the determinism principles in `docs/Engineering.md`.

### Phase 3: Verification Prediction Visibility
- [ ] Task 3.1: Modify the `latest-verify.json` schema (and corresponding Rust structs in `src/verify/`) to include a `warnings` or `prediction_diagnostics` field.
- [ ] Task 3.2: Update `src/commands/verify.rs` and `src/verify/predict.rs` so that when `PredictionResult` contains warnings, they are mapped into the report struct.
- [ ] Task 3.3: Ensure storage and history load failures in `src/commands/verify.rs` (currently using `.ok()` and `.unwrap_or_default()`) populate these warnings instead of silently returning empty histories and masking the degradation.

### Phase 4: Quality Assurance
- [ ] Task 4.1: Run `cargo check --all-features` and `cargo test --all-features` to ensure no functionality is broken by the formatting and error visibility changes.
- [ ] Task 4.2: Audit the git diff to ensure no `unwrap()` or `expect()` calls were added in production code during remediation.
# Track U1 Plan: Single Integration Test Harness

- [ ] Task U1.1: Declare unified integration test target in `Cargo.toml`.
- [ ] Task U1.2: Create the integration test main harness at `tests/integration/main.rs`.
- [ ] Task U1.3: Port each standalone integration test (`cli_doctor.rs`, `cli_impact.rs`, etc.) to submodules inside `tests/integration/`.
- [ ] Task U1.4: Remove old standalone test files from `tests/`.
- [ ] Task U1.5: Validate that `cargo nextest run --workspace` passes cleanly without WDAC blocks.

## Plan: State Migrations Decomposition
### Phase 1: Modular Setup & Test Harness
- [ ] Task 1.1: Create `src/state/migrations/mod.rs` and the new directory structure.
- [ ] Task 1.2: Write a snapshot or integration test that asserts the output of the current `Migrations::all()` to ensure no regressions in schema (M1-M15).
### Phase 2: Refactoring Migrations
- [ ] Task 2.1: Extract M1-M5 into `src/state/migrations/m1_to_m5.rs`.
- [ ] Task 2.2: Extract M6-M10 into `src/state/migrations/m6_to_m10.rs`.
- [ ] Task 2.3: Extract M11-M15 into `src/state/migrations/m11_to_m15.rs`.
- [ ] Task 2.4: Update `src/state/migrations/mod.rs` to assemble and export the complete `Migrations::all()` list for the orchestrator.
### Phase 3: Cleanup and Verification
- [ ] Task 3.1: Remove the old `src/state/migrations.rs` file and update all `mod.rs` references.
- [ ] Task 3.2: Run the test suite and verify no `unwrap()` calls exist and `miette` is properly used for error handling.
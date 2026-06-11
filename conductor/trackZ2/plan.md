# Track Z2 Plan: `data-models impact --changed` Clean-Tree Message

## Phase 1 — Red (Failing Tests)
- [ ] 1. Write an integration/unit test in `tests/integration/` or `src/commands/data_models.rs` that calls `data-models impact --changed` on a clean tree with indexed data models.
- [ ] 2. Assert that it fails to show `"No changed data models found."` (currently shows the help message).

## Phase 2 — Implementation
- [ ] 3. Modify `src/commands/data_models.rs`:
  - Calculate `total_models` by counting rows during the `data_models` loop, or query it.
  - In `impacted.is_empty()` check:
    - If `total_models > 0` and `changed` is true: print `"  No changed data models found."`
    - Else: print the original long help warning.

## Phase 3 — Green + Cleanup
- [ ] 4. Run `cargo nextest run --lib --bins --workspace` to verify.
- [ ] 5. Run clippy and format checks.

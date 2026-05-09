# Track R1-3: State Migrations Decomposition

## Objective
Decompose the monolithic `src/state/migrations.rs` (100KB) into a modular directory structure under `src/state/migrations/` without breaking existing schema migrations (M1-M15). Keep the public interface consistent.

## Requirements
- Move `src/state/migrations.rs` to a module directory `src/state/migrations/`.
- Group migrations logically (e.g., one file per major version or functional group, like `m1_to_m5.rs`, `m6_to_m10.rs`, `m11_to_m15.rs`).
- Implement `Migrations::all()` or an equivalent public interface in `src/state/migrations/mod.rs` to ensure the orchestrator remains unaffected.
- **Zero regressions** in existing schema migrations (M1-M15).
- Follow project standards: `miette` for errors, zero `unwrap()`, TDD methodology.

## Design Details
- `src/state/migrations/mod.rs`: Exports the unified list of all migrations.
- `src/state/migrations/m1_to_m5.rs` (or similar grouping): Contains actual SQL definitions and `Migration` struct constructions for M1 through M5.
- Tests should verify that `Migrations::all()` returns the exact same sequence and SQL as before.
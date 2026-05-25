# Track SE1: SQLite Storage Migration

## Status
Completed

## Milestone
SE: SQLite Storage Migration

## Problem
ChangeGuard currently defaults to Sled for CozoDB storage, which is marked as experimental upstream, has concurrency file-locking limitations on Windows, and has a bug in its deletion implementation that breaks AST incremental syncing and vector pruning (causing `test_incremental_graph_consistency` to be ignored). SQLite is already bundled and used for `ledger.db`, so migrating CozoDB to SQLite is the most robust, stable, and unified storage path.

## Objective
Migrate ChangeGuard's default CozoDB backend from Sled to SQLite, increment the crate version to `0.1.1`, un-ignore and verify the incremental graph consistency tests, and update documentation.

## Scope
- Update `Cargo.toml` dependencies to replace `cozo`'s `storage-sled` feature with `storage-sqlite` and `storage-sqlite-src`.
- Update the engine selection logic in `src/state/cozo/init.rs` to target `sqlite` instead of `sled`.
- Quiet `sqlite` instead of `sled` in the default log filter in `src/main.rs`.
- Update comments in `src/commands/update.rs`.
- Un-ignore `test_incremental_graph_consistency` in `tests/incremental_graph_consistency.rs`.
- Increment the package version to `0.1.1` in `Cargo.toml`.
- Update documentation.

## Non-Goals
- Do not migrate `ledger.db` schema (which is already native SQLite).
- Do not keep `storage-sled` as a dependency.

## Success Criteria
- [x] ChangeGuard compiles with Cozo SQLite backend.
- [x] Knowledge Graph files are created as SQLite database files (e.g., `ledger.cozo` as a SQLite file).
- [x] `test_incremental_graph_consistency` runs and passes successfully.
- [x] Crate version is bumped to `0.1.1`.
- [x] All tests in `changeguard` pass.

## Definition of Done
- [x] Dependency updated in `Cargo.toml` to `storage-sqlite` and `storage-sqlite-src` for `cozo`.
- [x] Crate version incremented to `0.1.1`.
- [x] `src/state/cozo/init.rs` updated to initialize `sqlite` engine.
- [x] `src/main.rs` default filter changed to `sqlite=warn`.
- [x] `tests/incremental_graph_consistency.rs` un-ignored.
- [x] `cargo test` runs and all tests pass.
- [x] Documentation updated to reflect SQLite migration.
- [x] Codex review completed.

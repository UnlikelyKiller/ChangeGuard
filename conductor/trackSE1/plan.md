# Track SE1 Plan: SQLite Storage Migration

## Phase 1: Dependency & Engine Configuration
- [x] Update `cozo` dependency features in `Cargo.toml` to replace `storage-sled` with `storage-sqlite` and `storage-sqlite-src`.
- [x] Increment crate version to `0.1.1` in `Cargo.toml`.
- [x] Modify `src/state/cozo/init.rs` to set engine to `sqlite` and update the lock retries and validation logic to use `sqlite`.
- [x] Update `src/main.rs` to filter `sqlite=warn` instead of `sled=warn`.
- [x] Update comment in `src/commands/update.rs`.

## Phase 2: Un-ignore and Verify Consistency Test
- [x] Remove `#[ignore = "..."]` attribute from `test_incremental_graph_consistency` in `tests/incremental_graph_consistency.rs`.
- [x] Run `cargo test` specifically on `test_incremental_graph_consistency` to verify that the SQLite engine correctly handles deletes/row updates and the consistency check passes.

## Phase 3: Final Verification & Documentation
- [x] Run full test suite: `cargo test --workspace`.
- [x] Run `changeguard verify` or cargo clippy/fmt checks to ensure no lint warnings.
- [x] Update project documentation to reflect the new SQLite default backend.
- [x] Conduct Codex review of the changes.

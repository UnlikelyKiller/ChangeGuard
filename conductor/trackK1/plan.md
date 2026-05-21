# Track K1: Storage Resilience Plan

## Phase 1: Tantivy Hardening
- [ ] Inspect `src/search/tantivy_engine.rs` indexing logic.
- [ ] Add explicit `commit()` and `wait_for_merges()` in `StreamIndexer::index_files`.
- [ ] Add logging for segment count after commit.
- [ ] Add integration test: index a directory and verify segment files exist on disk.

## Phase 2: CozoDB Cleanup Robustness
- [ ] Inspect `src/commands/update.rs` hard migration logic.
- [ ] Ensure `StorageManager` is dropped (closing SQLite and Cozo) before `fs::remove_dir_all`.
- [ ] Implement a `robust_remove_dir` utility that retries on "Permission Denied" (lock) for 1s.
- [ ] Add integration test: loop migration + re-indexing 5 times to confirm no corruption.

## Phase 3: Verification
- [ ] Run `changeguard index` on the repo itself and verify search matches.
- [ ] Run `changeguard update --migrate --force` and verify `index --semantic` succeeds.
- [ ] CI Gate: `cargo fmt && cargo clippy && cargo test`.

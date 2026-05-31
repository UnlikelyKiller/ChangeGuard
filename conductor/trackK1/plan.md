# Track K1: Storage Resilience Plan

## Phase 1: Tantivy Hardening & Verification
- [x] Inspect `src/search/tantivy_engine.rs` indexing logic.
- [x] Add explicit `commit()` and `wait_for_merges()` in `StreamIndexer::index_files`.
- [x] Implement `verify_index_segments` utility: count `.seg` files and compare with `meta.json`.
- [x] Add logging for segment count after commit.
- [x] Add integration test: index a directory and verify segment files exist on disk.

## Phase 2: CozoDB Cleanup & Integrity
- [x] Add `StorageManager::shutdown()` and `CozoStorage::shutdown()`.
- [x] Ensure shutdown is called before `update --migrate` wipes directories.
- [x] Implement `robust_remove_dir` utility that retries on "Permission Denied" (lock) for 1s.
- [x] Add "Cold Start Verification": run `::hnsw list` on first init to catch metadata corruption early.
- [x] Add integration test: loop migration + re-indexing 10 times to confirm no corruption.

## Phase 3: Final Verification
- [x] Run `changeguard index` on the repo itself and verify search matches.
- [x] Run `changeguard update --migrate --force` and verify `index --semantic` succeeds.
- [x] CI Gate.

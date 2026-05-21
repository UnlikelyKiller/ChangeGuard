# Track K1: Storage Resilience Plan

## Phase 1: Tantivy Hardening & Verification
- [ ] Inspect `src/search/tantivy_engine.rs` indexing logic.
- [ ] Add explicit `commit()` and `wait_for_merges()` in `StreamIndexer::index_files`.
- [ ] Implement `verify_index_segments` utility: count `.seg` files and compare with `meta.json`.
- [ ] Add logging for segment count after commit.
- [ ] Add integration test: index a directory and verify segment files exist on disk.

## Phase 2: CozoDB Cleanup & Integrity
- [ ] Add `StorageManager::shutdown()` and `CozoStorage::shutdown()`.
- [ ] Ensure shutdown is called before `update --migrate` wipes directories.
- [ ] Implement `robust_remove_dir` utility that retries on "Permission Denied" (lock) for 1s.
- [ ] Add "Cold Start Verification": run `::hnsw list` on first init to catch metadata corruption early.
- [ ] Add integration test: loop migration + re-indexing 10 times to confirm no corruption.

## Phase 3: Final Verification
- [ ] Run `changeguard index` on the repo itself and verify search matches.
- [ ] Run `changeguard update --migrate --force` and verify `index --semantic` succeeds.
- [ ] CI Gate.

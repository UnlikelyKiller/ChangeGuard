# Track K1: Storage Resilience (Windows & Vector Integrity)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
1. **Tantivy Persistence**: On Windows, the `index` command completes without error, but the `search_index/` directory occasionally lacks segment files despite a valid `meta.json`.
2. **CozoDB HNSW Corruption**: Hard state migrations fail to fully clear HNSW metadata due to file locks or incomplete relation dropping, leading to "Invalid neighbor degree" errors.

## Solution
1. **Tantivy**:
    - Ensure `index_writer.commit()` is called and its result is verified.
    - Implement `index_writer.wait_for_merges()` before dropping.
    - **New**: Implement post-index segment verification: check for `.seg` files on disk matching `meta.json`.
2. **CozoDB**:
    - **New**: Implement an explicit `shutdown()` routine for `StorageManager` to close all SQLite and Cozo handles.
    - Implement a `robust_remove_dir` utility with retry-on-lock logic for Windows.
    - **New**: Add a "Cold Start Validation" that runs `::hnsw list` and verifies metadata integrity on first init.

## Definition of Done (DoD)
- [ ] `changeguard index` results in visible `.seg` files in `.changeguard/search_index/` on Windows.
- [ ] `changeguard search -r ".*"` returns matches after indexing.
- [ ] `changeguard update --migrate --force` followed by `index --semantic` works every time without "neighbor degree" errors.
- [ ] Integration test: 10x loop of hard migration + re-indexing passes on Windows.
- [ ] CI gate passes.

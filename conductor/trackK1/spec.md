# Track K1: Storage Resilience (Windows & Vector Integrity)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
1. **Tantivy Persistence**: On Windows, the `index` command completes without error, but the `search_index/` directory contains only `meta.json` and no segment files. Consequently, `search` returns zero results.
2. **CozoDB HNSW Corruption**: Hard state migrations (`update --migrate --force`) occasionally fail to fully clear HNSW metadata. Subsequent `index --semantic` runs fail with `CozoDB script error: Invalid neighbor degree (metadata too short or corrupted)`.

## Solution
1. **Tantivy**:
    - Ensure `index_writer.commit()` is called and its result is checked.
    - Implement `index_writer.wait_for_merges()` before dropping.
    - Verify filesystem permissions and locking on Windows.
2. **CozoDB**:
    - Ensure the `DbInstance` is explicitly dropped before `Remove-Item` attempts to delete the `.cozo` file.
    - Add a retry loop for file deletion to handle Windows "file in use" locks.
    - Verify vector relation cleanup on first init.

## Definition of Done (DoD)
- [ ] `changeguard index` results in visible `.seg` files in `.changeguard/search_index/` on Windows.
- [ ] `changeguard search -r ".*"` returns matches after indexing.
- [ ] `changeguard update --migrate --force` followed by `index --semantic` works every time without "neighbor degree" errors.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.

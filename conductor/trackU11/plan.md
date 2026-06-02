# Track U11 Plan: Dynamic HNSW Rebuild Threshold Configuration

- [x] Task U11.1: Extend the configuration schema in `src/config/` to support `hnsw_rebuild_threshold` in the semantic block.
- [x] Task U11.2: Pass the threshold parameter from loaded configuration down to the `VectorStore::index_chunks` execution path.
- [x] Task U11.3: Update tests to verify that configuring different thresholds influences whether index rebuilding triggers.

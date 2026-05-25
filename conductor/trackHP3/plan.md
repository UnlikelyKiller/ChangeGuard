# Plan: Track HP3 (Cached Vector Nodes & Incremental HNSW Appends)

- [ ] 1. Design the binary storage format for HNSW graph nodes and edges under `.changeguard/state/vector.hnsw`.
- [ ] 2. Implement graph serialization and deserialization routines.
- [ ] 3. Update the indexing engine to perform incremental updates, inserting only modified/new symbol nodes.
- [ ] 4. Add deletion pruning to clean up stale nodes when files/symbols are removed.
- [ ] 5. Write unit and integration tests verifying incremental search correctness.

# Track U8 Plan: HNSW Build Speed Optimization

- [x] Task U8.1: Profile the current `index_chunks_batched` call to isolate bottlenecks in HNSW creation.
- [x] Task U8.2: Adjust threshold for dropping/rebuilding HNSW index based on repository scale.
- [x] Task U8.3: Implement optimized Cozo script calls to incrementally add vectors to the HNSW index rather than rebuilding it completely.
- [x] Task U8.4: Verify build behavior with threshold tests covering small batches and 500+ chunk rebuilds.

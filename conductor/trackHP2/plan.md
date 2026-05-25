# Plan: Track HP2 (Parallelized AST Chunk Ingestion & Embedding Generation)

- [ ] 1. Integrate `rayon` iteration inside `src/index/orchestrator.rs` or `src/index/walker.rs` to process files in parallel.
- [ ] 2. Implement a concurrent batch collector for snippet embedding generation.
- [ ] 3. Ensure sqlite and CozoDB connection pools are safely used across threads.
- [ ] 4. Benchmark performance and verify correct insertion of snippets.

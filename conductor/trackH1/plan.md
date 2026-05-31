# Plan: Track H1 (Semantic Engine Audit)

- [ ] 1. Debug `NaN` issue: Add tracing to the cosine similarity function and run a semantic search on a small repo.
- [ ] 2. Fix similarity math: Ensure safety checks for zero-magnitude vectors and proper float handling.
- [ ] 3. Audit BM25 scoring: Check the Tantivy index configuration for the search command.
- [ ] 4. Calibrate BM25: Tune the search parameters to provide better visual feedback for small-corpus results.
- [ ] 5. Implement regression tests for the semantic math layer.

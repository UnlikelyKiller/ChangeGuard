## Plan: Search Noise Reduction & Scoped Viz
### Phase 1: Search Pre-filtering
- [x] Task 1.1: Update `src/search/tantivy_engine.rs` to incorporate trigram index constraints.
- [x] Task 1.2: Adjust queries to prune documents failing the trigram threshold before BM25.
### Phase 2: Viz Scoping Flags
- [x] Task 2.1: Add `--limit`, `--depth`, and `--entity` arguments to `VizArgs` in `src/commands/viz.rs`.
- [x] Task 2.2: Update the knowledge graph query logic in `src/commands/viz.rs` to respect these bounds using recursive CozoDB reachability.
### Phase 3: Verification
- [x] Task 3.1: Run a search with common noisy terms and verify reduced output.
- [x] Task 3.2: Run `changeguard viz --entity main --depth 1` and verify the constrained subgraph.
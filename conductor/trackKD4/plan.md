## Plan: Track KD4 - PageRank-Based Churn & Centrality Risk Scoring

### Phase 1: Native PageRank Execution
- [x] Task 1.1: Design and test CozoScript that runs native PageRank:
  ```datalog
  edges[src, dst] := *edge{source: src, target: dst}
  ?[node, rank] <~ PageRank(edges[src, dst])
  ```
- [x] Task 1.2: Integrate PageRank calculation inside `src/index/centrality.rs` or run it during graph analysis in `changeguard index`.

### Phase 2: Risk Scoring Integration
- [x] Task 2.1: Retrieve the PageRank centrality scores during `ImpactOrchestrator` execution.
- [x] Task 2.2: Blend PageRank score with raw complexity and frequency metrics in `src/impact/hotspots.rs` using a normalized weighting factor.

### Phase 3: Verification
- [x] Task 3.1: Verify PageRank calculations on test repositories.
- [x] Task 3.2: Confirm risk scores update correctly and are sorted deterministically.

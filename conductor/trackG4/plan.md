## Plan: Track G4 Semantic Impact Orchestration

### Phase 1: KG Enrichment Provider
- [ ] **Task 1.1**: Create `src/impact/enrichment/kg_provider.rs` implementing the `EnrichmentProvider` trait.
- [ ] **Task 1.2**: Implement the `enrich` method to query CozoDB for nodes within 2 semantic hops of changed files.
- [ ] **Task 1.3**: Define a `SemanticImpact` struct in `src/impact/packet.rs` to hold conceptual neighbors and community labels.

### Phase 2: Datalog Reachability Queries
- [ ] **Task 2.1**: Implement a Datalog query to find "Conceptual Neighbors": nodes linked via `rationale_for` or `semantically_similar` edges.
- [ ] **Task 2.2**: Implement a query to identify the "Architectural Domain" (Community) of a changed file.
- [ ] **Task 2.3**: Integrate `KGProvider` into the `ImpactOrchestrator` execution flow.

### Phase 3: Verification (TDD)
- [ ] **Task 3.1**: Write a test verifying that changing a file in the "Auth" community returns other "Auth" nodes as semantic impact.
- [ ] **Task 3.2**: Write a test verifying that a documentation change can trigger a "Code Review Recommended" warning for semantically linked source files.
- [ ] **Task 3.3**: Verify that impact packets correctly serialize the new semantic enrichment data.

### Definition of Done (DoD)
- [x] `changeguard impact` includes semantic neighbors in its output.
- [x] Reachability queries successfully traverse both structural (code) and semantic (doc) edges.
- [x] No more than 4 files modified: `src/impact/enrichment/kg_provider.rs`, `src/impact/orchestrator.rs`, `src/impact/packet.rs`, `src/impact/enrichment/mod.rs`.

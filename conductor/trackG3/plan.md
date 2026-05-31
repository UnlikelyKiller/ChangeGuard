## Plan: Track G3 Graph Ingestion & Provenance

### Phase 1: Graph JSON Parser
- [ ] **Task 1.1**: Create `src/index/graph_loader.rs` and define serializable structs for `graph.json` nodes and edges.
- [ ] **Task 1.2**: Implement `load_from_file(path: &Path)` to read and deserialize the graphify output.
- [ ] **Task 1.3**: Implement a helper to correlate graph node IDs with existing `project_symbols` IDs in the database.

### Phase 2: Bulk Loading & Provenance
- [ ] **Task 2.1**: Implement `CozoStorage::insert_graph_nodes(nodes: Vec<GraphNode>)` using batch Datalog inserts.
- [ ] **Task 2.2**: Implement `CozoStorage::insert_graph_edges(edges: Vec<GraphEdge>)` ensuring `provenance_id` is linked to the latest ledger transaction.
- [ ] **Task 2.3**: Integrate `graph_loader` into `src/commands/index.rs`: after a successful `graphify` run, ingest the resulting JSON.

### Phase 3: Verification (TDD)
- [ ] **Task 3.1**: Write a test that ingests a sample `graph.json` (3 nodes, 2 edges) and verifies they exist in CozoDB.
- [ ] **Task 3.2**: Verify that nodes are correctly tagged with the `EXTRACTED` or `INFERRED` confidence levels from the JSON.
- [ ] **Task 3.3**: Verify that orphaned edges (missing source/target) are safely ignored or logged as warnings.

### Definition of Done (DoD)
- [x] `graphify-out/graph.json` is successfully ingested into CozoDB.
- [x] Graph nodes are linked to their corresponding symbol provenance in the ledger.
- [x] No more than 4 files modified: `src/index/graph_loader.rs`, `src/index/mod.rs`, `src/commands/index.rs`, `src/state/storage/cozo.rs`.

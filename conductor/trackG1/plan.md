## Plan: Track G1 CozoDB Integration & Schema

### Phase 1: Dependency & Wrapper
- [ ] **Task 1.1**: Add `cozo = { version = "0.7", features = ["storage-sqlite"] }` to `Cargo.toml`.
- [ ] **Task 1.2**: Create `src/state/storage/cozo.rs` and define the `CozoStorage` struct with `DbInstance`.
- [ ] **Task 1.3**: Implement `CozoStorage::new(path: &Path)` to initialize a persistent SQLite-backed Cozo instance.
- [ ] **Task 1.4**: Implement `CozoStorage::run_script(&self, script: &str)` as a safe wrapper around `db.run_script`.

### Phase 2: Datalog Schema Definition
- [ ] **Task 2.1**: Define the `:create node` relation in `cozo.rs` (id, label, category, risk_score, metadata).
- [ ] **Task 2.2**: Define the `:create edge` relation in `cozo.rs` (source, target, relation, confidence, provenance_id).
- [ ] **Task 2.3**: Define the `:create ledger_link` relation (node_id, ledger_id, interaction_type).
- [ ] **Task 2.4**: Implement a `setup_schema()` method that runs these definitions on startup.

### Phase 3: Verification (TDD)
- [ ] **Task 3.1**: Write a unit test in `cozo.rs` that initializes an in-memory Cozo instance and verifies relations exist via `::relations`.
- [ ] **Task 3.2**: Write a test verifying that `node` and `edge` relations can handle bulk inserts of sample graph data.
- [ ] **Task 3.3**: Write a test for a basic Datalog reachability query: "Find all nodes reachable from Node A".

### Definition of Done (DoD)
- [x] CozoDB is initialized without errors.
- [x] All 3 core relations (`node`, `edge`, `ledger_link`) are queryable.
- [x] Reachability test passes with a 2-hop sample graph.
- [x] No more than 4 files modified: `Cargo.toml`, `src/state/storage/mod.rs`, `src/state/storage/cozo.rs`.

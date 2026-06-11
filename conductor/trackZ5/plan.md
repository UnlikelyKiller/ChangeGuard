# Track Z5 Plan: Test Mapping Graph Loader

## Phase 1 — Red (Failing Tests)
- [ ] 1. Write an integration/unit test in `tests/integration/` or `src/commands/test_mapping.rs` that populates `test_mapping` in SQLite, runs `build_native_graph`, and verifies the `validates` edges are present in CozoDB.
- [ ] 2. Currently, the test will fail since `test_mapping` is not loaded into CozoDB.

## Phase 2 — Implementation
- [ ] 3. Add a `phase_test_mappings` function to `src/index/graph_loader.rs`:
  - Query all rows from the SQLite `test_mapping` table.
  - For each mapping row, resolve the URN for the test symbol and the tested symbol/file.
  - Insert `GraphNode` for the test symbol with `category = NodeKind::Test` and URN `urn:changeguard:test:{name}`.
  - Create `GraphEdge` from the test URN to the tested URN with `relation = EdgeKind::Validates` (which corresponds to `'validates'` in Datalog).
- [ ] 4. Invoke `phase_test_mappings` in `build_native_graph` in `src/index/graph_loader.rs`.

## Phase 3 — Green + Cleanup
- [ ] 5. Run `cargo nextest run --lib --bins --workspace` and verify the tests pass.

# Track Z6 Plan: Ledger Graph Transaction Edges

## Phase 1 — Red (Failing Tests)
- [ ] 1. Write a test `tests/integration/ledger_graph_edges.rs::test_commit_writes_kg_edges`: start a transaction, edit a file, commit, and assert CozoDB contains the `affects` edge from transaction URN to file URN.
- [ ] 2. Currently, the test will fail as no edges are written.

## Phase 2 — Implementation
- [ ] 3. In `TransactionManager::commit_transaction` in `src/ledger/transaction.rs`:
  - Fetch the list of files modified during the transaction from the SQLite ledger database.
  - If CozoDB is initialized, construct the transaction URN (`urn:changeguard:transaction:{tx_id}`) and file URN (`urn:changeguard:file:{relative_path}`) for each changed file.
  - Insert these edges into CozoDB with relation `'affects'` (or `EdgeKind::Affects` equivalent).
  - Gate edge writing on `storage.cozo.is_some()`. If writing fails, log a warning but do not fail the commit.

## Phase 3 — Green + Cleanup
- [ ] 4. Run `cargo nextest run --lib --bins --workspace` and verify the test passes.

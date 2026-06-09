# Track X4 Plan: `ledger graph` Transaction→Entity Edges

## Phase 1 — Red (Failing Tests)
- [x] 1. Write integration test `tests/integration/ledger_graph_edges.rs::test_commit_writes_kg_edges`: start tx, commit with changed files list, assert CozoDB has `edge` rows where `source = tx_urn`.
- [x] 2. Write unit test for URN construction: given a relative file path, assert correct `urn:changeguard:file:...` output.

## Phase 2 — Implementation
- [x] 3. Identify where changed files are captured during `ledger commit`. In `src/commands/ledger.rs` `execute_ledger_commit`, after the SQLite commit succeeds:
  - Open CozoDB storage (or use the existing `StorageManager`).
  - Build the transaction URN: `format!("urn:changeguard:transaction:{}", full_tx_id)`.
- [x] 4. Retrieve the list of files changed in this transaction from `TransactionManager::get_transaction_files(tx_id)` (or equivalent query on `project_file_changes` SQLite table).
- [x] 5. For each changed file path, build a `GraphEdge`:
  ```rust
  GraphEdge {
      source: tx_urn.clone(),
      target: format!("urn:changeguard:file:{}", relative_path),
      relation: EdgeKind::Affects.to_string(),
      metadata: None,
  }
  ```
- [x] 6. Call `cozo.insert_edges(&edges)` wrapped in a `if let Some(cozo) = &storage.cozo { ... }` guard.
- [x] 7. On error, emit `warn!("ledger graph: failed to write KG edges: {}", e)` and continue (SQLite commit already done — do not unwind).
- [x] 8. Also ensure the `LedgerTransaction` node is upserted: `node{id: tx_urn, label: tx_id, category: "ledger_transaction", metadata: {...}}`.

## Phase 3 — Green + Cleanup
- [x] 9. Verify locally: `changeguard ledger graph <recent-tx-id>` shows file rows.
- [x] 10. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 11. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 12. Run `cargo fmt --all -- --check` — clean.
- [x] 13. Update `conductor/conductor.md` status to Completed.

## Phase 4 — Codex Review Remediation
- [x] 14. Fix `execute_ledger_adopt`: `adopt_drift()` now returns `Vec<String>` of promoted tx_ids; files are collected from all adopted drift transactions for real KG `Affects` edges (replacing synthetic `"drift_adoption"` entity).
- [x] 15. Fix silent error swallowing: `get_transaction_files` errors are now logged via `tracing::warn!` in `execute_ledger_commit`, `execute_ledger_adopt`, and `execute_ledger_atomic`.
- [x] 16. Added `test_adopt_writes_kg_edges_with_real_files` integration test verifying finding #1 resolution.
- [x] 17. Full verify: 759/759 unit tests, 209/209 integration tests pass. fmt + clippy clean. Committed `4422dea`.


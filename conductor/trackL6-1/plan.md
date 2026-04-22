# Plan: Track L6-1: Ledger Federation

### Phase 1: Schema and Data Types
- [ ] Task 1.1: Create `FederatedLedgerEntry` struct in `src/federated/schema.rs` with necessary fields (e.g., `tx_id`, `category`, `entry_type`, `entity`, `change_type`, `summary`, `reason`, `is_breaking`, `committed_at`).
- [ ] Task 1.2: Add `pub ledger: Option<Vec<FederatedLedgerEntry>>` to `FederatedSchema`.
- [ ] Task 1.3: Update `FederatedSchema::validate()` to ensure `ledger` entries do not contain path traversal elements (`..` or absolute paths).

### Phase 2: Ledger Federation Logic
- [ ] Task 2.1: Create `src/ledger/federation.rs`.
- [ ] Task 2.2: Implement `export_ledger_entries(conn: &Connection, days: i64) -> Result<Vec<FederatedLedgerEntry>>` to fetch recent `LOCAL` entries.
- [ ] Task 2.3: Implement `import_federated_entries(conn: &Connection, sibling_name: &str, entries: &[FederatedLedgerEntry]) -> Result<()>` to insert stub transactions and federated `ledger_entries` (with `origin='SIBLING'` and `trace_id=sibling_name`).

### Phase 3: CLI Command Integration
- [ ] Task 3.1: Update `src/commands/federate.rs` -> `execute_federate_export` to call `export_ledger_entries` and populate the schema.
- [ ] Task 3.2: Update `src/commands/federate.rs` -> `execute_federate_scan` to process `schema.ledger` and invoke `import_federated_entries`.

### Phase 4: Cross-Repo Impact and UX
- [ ] Task 4.1: Update `src/federated/impact.rs` to query `ledger_entries` for `SIBLING` entries matching the dependent `sibling_file` and `sibling_name`.
- [ ] Task 4.2: Append clear warnings to `impact_reasons` for modified sibling entities (e.g., `Cross-repo impact: Sibling X modified Y ([FEDERATED] <summary>)`).
- [ ] Task 4.3: Update search and status rendering (e.g., `src/ledger/search.rs` or `src/commands/ledger_search.rs`) to prefix federated output with `[FEDERATED]`.

### Phase 5: Testing and Polish
- [ ] Task 5.1: Create `tests/federated_ledger.rs` to verify export/import logic and path confinement security.
- [ ] Task 5.2: Create an integration test validating `federate scan` and subsequent `impact` output with federated ledger entries.
- [ ] Task 5.3: Run `cargo clippy`, `cargo fmt`, and `cargo test` to ensure stability and verify all tests pass.

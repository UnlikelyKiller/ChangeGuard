# Plan: Track L6-R: Ledger Federation Remediation

### Phase 1: Schema and Domain Logic Alignment
- [ ] Task 1.1: Edit `src/federated/schema.rs` to remove the `origin` field from the `FederatedLedgerEntry` struct.
- [ ] Task 1.2: Edit `src/ledger/federation.rs` to adjust the `FederatedLedgerEntry` mapping in `export_ledger_entries` and `import_federated_entries` based on the schema change.
- [ ] Task 1.3: Update `src/ledger/federation.rs` to enforce the 30-day export limit in `export_ledger_entries` (using RFC3339 string comparison on `committed_at`).

### Phase 2: Security and Import Integrity
- [ ] Task 2.1: Update `src/ledger/federation.rs` `import_federated_entries` to enforce strict path confinement. Reject paths with `..`, absolute paths, or UNC roots.
- [ ] Task 2.2: Update `src/ledger/federation.rs` `import_federated_entries` to correctly store `origin = 'SIBLING'` and `trace_id = sibling_name` in the database.
- [ ] Task 2.3: Update the duplicate check in `import_federated_entries` to correctly key on the new `origin` and `trace_id` values.

### Phase 3: Integration and UX Corrections
- [ ] Task 3.1: Edit `src/federated/impact.rs` to query the local ledger database for federated entries (`origin='SIBLING'`, `trace_id=sibling_name`, `entity_normalized=sibling_file`) instead of reading directly from `schema.ledger`.
- [ ] Task 3.2: Edit `src/commands/ledger_audit.rs` to accurately display the `[FEDERATED: {trace_id}]` tag only when `origin == "SIBLING"`.

### Phase 4: Test Alignment and Verification
- [ ] Task 4.1: Edit `tests/ledger_federation.rs` to fix the `execute_scan` signature to match the current API: `execute_scan(run_impact: bool)`.
- [ ] Task 4.2: Run and verify all unit tests, integration tests, and ensure code compiles successfully.

## Plan: Track L2-2 Ledger Reconciliation & Adoption
### Phase 1: Core Transaction Logic
- [ ] Task 1.1: Add `reconcile_unaudited` and `adopt_unaudited` to `TransactionManager` in `src/ledger/transaction.rs`.
- [ ] Task 1.2: Add bulk `UNAUDITED` querying (by glob/pattern) and `auto_reconcile_entity` helper to `TransactionManager`.
- [ ] Task 1.3: Update `src/ledger/db.rs` with `get_unaudited_by_pattern` using SQLite `LIKE` or glob matching to support bulk queries.

### Phase 2: CLI Command Implementation
- [ ] Task 2.1: Implement `execute_ledger_reconcile` in `src/commands/ledger.rs` (or new dedicated module) mapping to the transaction backend. Handle `--tx-id`, `--entity-pattern`, and `--auto-reconcile`.
- [ ] Task 2.2: Implement `execute_ledger_adopt` in `src/commands/ledger.rs` mapping to the `adopt_unaudited` backend.
- [ ] Task 2.3: Modify `execute_ledger_commit` in `src/commands/ledger.rs` to support auto-reconciling matching `UNAUDITED` entries upon commit.

### Phase 3: Testing & Validation
- [ ] Task 3.1: Write integration tests in `tests/ledger_drift.rs` for individual reconciliation and adoption flows.
- [ ] Task 3.2: Write integration tests for bulk pattern reconciliation.
- [ ] Task 3.3: Write integration tests for auto-reconcile during commit.
- [ ] Task 3.4: Verify output formatting of `ledger status` reflects newly adopted and reconciled states accurately.

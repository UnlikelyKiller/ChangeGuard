## Plan: Track L1-2 - Transaction Lifecycle Management

### Phase 1: Storage Layer and Database Migrations
- [ ] Task 1.1: Create Migration 11 (`M11`) in `src/state/migrations.rs` to create the `transactions` table and necessary indexes.
- [ ] Task 1.2: Create Migration 12 (`M12`) in `src/state/migrations.rs` to create the `ledger_entries` table, the `ledger_fts` virtual table, and `AFTER INSERT/UPDATE/DELETE` triggers for FTS sync.
- [ ] Task 1.3: Create `src/ledger/db.rs` with helper methods for interacting with `transactions` and `ledger_entries` using `rusqlite`.
- [ ] Task 1.4: Update `src/config/model.rs` with `LedgerConfig` extension.

### Phase 2: Transaction Core and Session Management
- [ ] Task 2.1: Implement `src/ledger/session.rs` to provide the process startup timestamp as a session identifier.
- [ ] Task 2.2: Implement path normalization logic (`entity_normalized`) in `src/ledger/transaction.rs` or utility to conditionally case-fold based on the filesystem.
- [ ] Task 2.3: Implement `start_change`, `commit_change`, and `rollback_change` logic in `src/ledger/transaction.rs`.
- [ ] Task 2.4: Implement fuzzy UUID matching and retrieval logic for pending transactions in `transaction.rs`.

### Phase 3: CLI Commands Integration
- [ ] Task 3.1: Update `src/cli.rs` and `src/commands/mod.rs` to include the `ledger` command group and its enums.
- [ ] Task 3.2: Implement `src/commands/ledger.rs` for `ledger start`.
- [ ] Task 3.3: Implement `src/commands/ledger_commit.rs` for `ledger commit`.
- [ ] Task 3.4: Implement `src/commands/ledger_rollback.rs` for `ledger rollback`.
- [ ] Task 3.5: Implement `src/commands/ledger_atomic.rs` for `ledger atomic`.
- [ ] Task 3.6: Implement `src/commands/ledger_note.rs` for `ledger note`.
- [ ] Task 3.7: Implement `src/commands/ledger_status.rs` for `ledger status` (including pending vs stale identification).
- [ ] Task 3.8: Implement `src/commands/ledger_resume.rs` for `ledger resume`.

### Phase 4: Testing & Verification
- [ ] Task 4.1: Write `tests/ledger_lifecycle.rs` checking `start -> commit` roundtrip.
- [ ] Task 4.2: Add tests in `tests/ledger_lifecycle.rs` for `start -> rollback` and `atomic` operations.
- [ ] Task 4.3: Validate the `ledger note` restrictions and ghost commit guards in integration tests.
- [ ] Task 4.4: Verify fuzzy matching disambiguation handles both unique truncations and ambiguous duplicates correctly.
## Plan: Track L1-2 - Transaction Lifecycle Management

### Phase 1: Storage Layer and Database Migrations
- [x] Task 1.1: Ensure Migration 11 (`M11`) in `src/state/migrations.rs` is complete for `transactions` table.
- [x] Task 1.2: Ensure Migration 12 (`M12`) in `src/state/migrations.rs` is complete for `ledger_entries` and `ledger_fts`.
- [x] Task 1.3: Create `src/ledger/db.rs` with helper methods for interacting with `transactions` and `ledger_entries` using `rusqlite`.
- [x] Task 1.4: Update `src/config/model.rs` with `LedgerConfig` extension.

### Phase 2: Transaction Core and Session Management
- [x] Task 2.1: Implement `src/ledger/session.rs` to provide the process startup timestamp as a session identifier.
- [x] Task 2.2: Implement path normalization logic (`entity_normalized`) in `src/ledger/transaction.rs` or utility to conditionally case-fold based on the filesystem.
- [x] Task 2.3: Implement `start_change`, `commit_change`, and `rollback_change` logic in `src/ledger/transaction.rs`.
- [x] Task 2.4: Implement fuzzy UUID matching and retrieval logic for pending transactions in `transaction.rs`.

### Phase 3: CLI Commands Integration
- [x] Task 3.1: Update `src/cli.rs` and `src/commands/mod.rs` to include the `ledger` command group and its enums.
- [x] Task 3.2: Implement `src/commands/ledger.rs` for `ledger start`.
- [x] Task 3.3: Implement `src/commands/ledger_commit.rs` for `ledger commit`.
- [x] Task 3.4: Implement `src/commands/ledger_rollback.rs` for `ledger rollback`.
- [x] Task 3.5: Implement `src/commands/ledger_atomic.rs` for `ledger atomic`.
- [x] Task 3.6: Implement `src/commands/ledger_note.rs` for `ledger note`.
- [x] Task 3.7: Implement `src/commands/ledger_status.rs` for `ledger status` (including pending vs stale identification).
- [x] Task 3.8: Implement `src/commands/ledger_resume.rs` for `ledger resume`.

### Phase 4: Testing & Verification
- [x] Task 4.1: Write `tests/ledger_lifecycle.rs` checking `start -> commit` roundtrip.
- [x] Task 4.2: Add tests in `tests/ledger_lifecycle.rs` for `start -> rollback` and `atomic` operations.
- [x] Task 4.3: Validate the `ledger note` restrictions and ghost commit guards in integration tests.
- [x] Task 4.4: Verify fuzzy matching disambiguation handles both unique truncations and ambiguous duplicates correctly.
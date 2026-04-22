## Plan: Track L1-1 Ledger Data Model & Migrations

### Phase 1: Ledger Types & Errors
- [ ] Task 1.1: Create `src/ledger/types.rs` and define `Category`, `ChangeType`, `VerificationStatus`, `VerificationBasis`, and `EntryType` enums.
- [ ] Task 1.2: Add `serde` and `clap` attribute derivations to the enums in `src/ledger/types.rs` enforcing proper casing rules (e.g., `SCREAMING_SNAKE_CASE`, `snake_case`).
- [ ] Task 1.3: Write unit tests in `src/ledger/types.rs` to verify the JSON serialization and deserialization formats.
- [ ] Task 1.4: Create `src/ledger/error.rs` and define the `LedgerError` enum with `thiserror` and `miette::Diagnostic` derivations.
- [ ] Task 1.5: Write unit tests in `src/ledger/error.rs` confirming correct error formatting.
- [ ] Task 1.6: Update `src/ledger/mod.rs` to export `types` and `error` modules. Add `src/ledger` module to `src/lib.rs` (if not already present).

### Phase 2: Configuration Model Updates
- [ ] Task 2.1: In `src/config/model.rs`, define the `LedgerConfig`, `CategoryMapping`, and `WatcherPattern` structs.
- [ ] Task 2.2: Implement `Default` traits for the new structs, referencing the defaults specified in `docs/Ledger-Incorp-plan.md`.
- [ ] Task 2.3: Add a `pub ledger: LedgerConfig` field to the `Config` struct.
- [ ] Task 2.4: Update tests in `src/config/model.rs` to verify `LedgerConfig` default values and proper TOML deserialization.

### Phase 3: SQLite Database Migrations
- [ ] Task 3.1: In `src/state/migrations.rs`, add migration M11 to create the `transactions` table and related indices (`idx_transactions_entity_status`, `idx_transactions_status`, `idx_transactions_session_id`, `idx_transactions_operation_id`).
- [ ] Task 3.2: In `src/state/migrations.rs`, add migration M12 to create the `ledger_entries` table, `ledger_fts` virtual table, and FTS5 content-sync triggers.
- [ ] Task 3.3: Update `test_all_tables_exist` in `src/state/migrations.rs` to verify the presence of `transactions`, `ledger_entries`, and `ledger_fts`.
- [ ] Task 3.4: Write a new test in `src/state/migrations.rs` (`test_insert_and_query_ledger_transaction`) to verify writes and basic queries to the newly added tables.

## Plan: Track L5-1: Token-Level Provenance

### Phase 1: Data Model and Migrations
- [x] Task 1.1: Create `src/ledger/provenance.rs` and define `TokenProvenance` struct and `ProvenanceAction` enum (`Added`, `Modified`, `Deleted`).
- [x] Task 1.2: Add Migration M14 to `src/state/migrations.rs` to create the `token_provenance` table with necessary columns (`id`, `tx_id`, `entity`, `entity_normalized`, `symbol_name`, `symbol_type`, `action`) and indices.
- [x] Task 1.3: Write unit tests in `src/state/migrations.rs` to ensure M14 validates correctly and creates `token_provenance`.

### Phase 2: Core Logic and TransactionManager Integration
- [x] Task 2.1: Update `TransactionManager` or `src/ledger/db.rs` to handle inserting provenance records via a new method `record_token_provenance`.
- [x] Task 2.2: Add method `get_token_provenance_by_tx` and `get_token_provenance_by_entity` to retrieve provenance.
- [x] Task 2.3: Implement utility `compute_symbol_diff(old_symbols, new_symbols)` in `src/ledger/provenance.rs`, leveraging `Symbol` from `src/index/symbols.rs`.
- [x] Task 2.4: Write TDD integration tests in `tests/ledger_provenance.rs` for tracking token changes.

### Phase 3: CLI Commands
- [ ] Task 3.1: Create `src/commands/ledger_track.rs` for the `ledger track` command (Deferred: automated provenance preferred).
- [ ] Task 3.2: Register `ledger track` in `src/cli.rs` (Deferred).
- [x] Task 3.3: Update `TransactionManager` to support token provenance recording during lifecycle.
- [x] Task 3.4: Write integration tests verifying the token extraction logic.

### Phase 4: Output and Audit Updates
- [x] Task 4.1: Update `src/commands/ledger_audit.rs` to fetch and interleave `token_provenance` records when `--entity <path>` is provided, showing token-level history.
- [x] Task 4.2: Update `src/commands/ledger_status.rs` to display a brief summary or count of tracked symbols for transactions.
- [x] Task 4.3: Add final end-to-end test verifying `ledger start` -> `ledger track` -> `ledger commit` -> `ledger audit --entity` flow.

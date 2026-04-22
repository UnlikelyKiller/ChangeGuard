# Plan: Track L2-1 - Ledger Drift Detection

### Phase 1: Data Model & Database Support
- [ ] Task 1.1: Update `Transaction` struct in `src/ledger/types.rs` to include `detected_at`, `drift_count`, `first_seen_at`, and `last_seen_at`.
- [ ] Task 1.2: Update `insert_transaction`, `get_transaction`, and `get_pending_by_entity` in `src/ledger/db.rs` to select/insert the new fields.
- [ ] Task 1.3: Add `get_unaudited_by_entity` and `upsert_unaudited_transaction` to `src/ledger/db.rs`. The upsert should insert a new record or increment `drift_count` and update `last_seen_at` for an existing `UNAUDITED` record.

### Phase 2: Drift Manager Implementation
- [ ] Task 2.1: Create `src/ledger/drift.rs` and define the `DriftManager` struct.
- [ ] Task 2.2: Implement `DriftManager::process_event(path)`. It should check for `PENDING` transactions and, if missing, delegate to `db.upsert_unaudited_transaction`.
- [ ] Task 2.3: Add unit tests in `src/ledger/drift.rs` for deduplication and conditional ignoring of `PENDING` files.

### Phase 3: Watcher Integration
- [ ] Task 3.1: Update `src/commands/watch.rs` to instantiate `StorageManager` and `DriftManager` for the watcher's lifespan.
- [ ] Task 3.2: In the watch callback, invoke `DriftManager::process_event` for each modified file in the batch. Ensure errors are gracefully logged via `tracing::warn` without panicking the watcher thread.

### Phase 4: Ledger Status Enhancements
- [ ] Task 4.1: Update `src/ledger/db.rs` to add queries for fetching all `PENDING` and all `UNAUDITED` transactions.
- [ ] Task 4.2: Update `execute_ledger_status` in `src/commands/ledger.rs` to fetch and display `UNAUDITED` entries, including their `drift_count`.
- [ ] Task 4.3: Implement stale transaction detection in `execute_ledger_status` based on `started_at` and a 24-hour threshold, visually separating them from active `PENDING` transactions.

### Phase 5: Integration Testing
- [ ] Task 5.1: Create `tests/ledger_drift.rs`.
- [ ] Task 5.2: Write tests simulating untracked changes resulting in `UNAUDITED` creation.
- [ ] Task 5.3: Write tests verifying that multiple edits to the same file increment `drift_count` instead of creating multiple records.
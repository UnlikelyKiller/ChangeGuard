# Specification: Track L2-1 - Ledger Drift Detection

## 1. Overview
This track implements the foundation for Phase L2 (Drift Detection & Reconciliation) of the Ledger Incorporation Plan. It bridges the existing ChangeGuard file watcher with the transactional ledger system, automatically recording untracked filesystem changes as `UNAUDITED` transactions.

## 2. Deliverables

### 2.1. Data Model & DB Updates
Update `src/ledger/types.rs` to include the new fields for `UNAUDITED` transactions in the `Transaction` struct:
```rust
pub struct Transaction {
    // ... existing fields ...
    pub detected_at: Option<String>,
    pub drift_count: Option<i32>,
    pub first_seen_at: Option<String>,
    pub last_seen_at: Option<String>,
}
```
*Note: The `transactions` table schema already supports these from Track L1 migrations, but the Rust types and SQL queries in `src/ledger/db.rs` need to read/write them.*

**Updates in `src/ledger/db.rs`:**
- Update `insert_transaction` and `get_transaction` to map the new fields.
- Add `upsert_unaudited_transaction(entity_normalized, entity, timestamp)` to atomically insert a new `UNAUDITED` entry or increment `drift_count` and update `last_seen_at` if one already exists for the current session.
- Ensure SQLite indices (`idx_transactions_entity_status`) are used for fast lookup of `PENDING` vs `UNAUDITED` states.

### 2.2. Drift Manager (`src/ledger/drift.rs`)
Create a new module to process file watcher events.
```rust
pub struct DriftManager<'a> {
    db: &'a LedgerDb<'a>,
    session_id: String, // Watcher's session ID
}

impl<'a> DriftManager<'a> {
    pub fn process_event(&self, event_path: &str, timestamp: &str) -> Result<(), LedgerError>;
}
```
**Logic flow:**
1. Normalize the `event_path`.
2. Check if a `PENDING` transaction exists for `entity_normalized`. If yes, ignore (change is expected).
3. If no `PENDING` transaction exists, call `upsert_unaudited_transaction`. This creates a new `UNAUDITED` transaction (with `source: WATCHER`) or increments the `drift_count` of an existing `UNAUDITED` transaction for this file.

### 2.3. Watcher Integration (`src/commands/watch.rs`)
Integrate `DriftManager` into the `execute_watch` callback.
- When `execute_watch` starts, initialize a database connection and instantiate `DriftManager`.
- Inside the `batch` callback, iterate through `batch.events`.
- Filter events (using hardcoded high-signal lists or config).
- Call `drift_mgr.process_event` for relevant file modifications.
- **Concurrency:** The watcher runs asynchronously. Ensure `StorageManager::init` connection handles SQLite locking correctly (WAL mode + `busy_timeout` is already set in `StorageManager`).

### 2.4. Status Enhancements (`src/commands/ledger.rs`)
Update `execute_ledger_status` to differentiate between:
- **Active Session:** PENDING transactions started recently.
- **Stale Drift:** PENDING transactions older than `config.ledger.stale_threshold_hours` (default 24h). Add a prominent warning suggesting rollback or adoption.
- **Unaudited Changes:** Display `UNAUDITED` transactions and their `drift_count`.

## 3. Testing Strategy
- **Unit Tests:** Verify `DriftManager` correctly deduplicates sequential events for the same file into a single `UNAUDITED` record with `drift_count > 1`.
- **Integration Tests:** `tests/ledger_drift.rs` should simulate file changes using a mock watcher or direct `DriftManager` calls, verifying database state transitions.

## 4. Edge Cases
- **Rapid Saves:** The watcher already debounces events. `upsert_unaudited_transaction` provides a secondary deduplication layer at the DB level.
- **Concurrency:** DB connection must not panic on `SQLITE_BUSY`. `StorageManager`'s 5000ms `busy_timeout` handles this.
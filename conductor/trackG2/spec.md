# Specification: Track G2 Data Portability & Ledger Integration

## Goal
Unify the existing ChangeGuard state (Ledger, Symbols) into the **CozoDB** engine to enable complex cross-referencing between code changes and architectural relationships.

## Context
Currently, ChangeGuard uses a standard SQLite schema for the `ledger` and `project_symbols` tables. To perform advanced graph queries (e.g., "Find all symbols modified in the last 3 'Architecture' transactions"), we need this data in CozoDB.

## Technical Details

### 1. Schema Expansion
Extend the `CozoStorage::setup_schema` to include mirrored relations for the legacy SQLite tables:
- `ledger_entry`: (id, timestamp, author, message, status, metadata)
- `project_symbol`: (id, file_id, name, kind, location, metadata)

### 2. Migration Bridge (`src/state/migration/cozo_port.rs`)
Implement a robust porting utility that:
1.  Connects to the legacy `SqliteStorage`.
2.  Iterates through all tables.
3.  Converts rows into CozoDB-compatible JSON objects.
4.  Executes batch `put` commands in CozoDB.

### 3. State Initialization
Update `src/state/mod.rs` to:
- Initialize both SQLite and CozoDB backends.
- Detect if the CozoDB is empty while the SQLite DB is not.
- Trigger the migration bridge if needed.
- Log a successful migration audit entry.

## TDD Requirements
1.  **Parity Test**: Insert data into SQLite, run the migration, and verify that a Datalog query for symbols returns the same set as a SQL query.
2.  **Idempotency**: Ensure that running the migration multiple times doesn't create duplicate entries in CozoDB (using PK constraints).
3.  **Large Batch Test**: Verify that a ledger with 1,000+ entries can be migrated in under 2 seconds.

## Definition of Done
- [ ] Ledger and Symbol data is accessible via Datalog.
- [ ] Automated migration bridge is implemented and tested.
- [ ] Parity verification tests pass.
- [ ] No more than 4 files modified: `src/state/storage/cozo.rs`, `src/state/mod.rs`, `src/state/migration/mod.rs`, `src/state/migration/cozo_port.rs`.

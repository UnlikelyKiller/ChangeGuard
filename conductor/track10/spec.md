# Specification: Track 10 - State SQLite Persistence

## Overview
Implement a persistent storage layer using `rusqlite` to store repository snapshots and historical impact packets in `.changeguard/state/ledger.db`.

## Schema

### Snapshots Table
- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `timestamp`: TEXT (ISO 8601)
- `head_hash`: TEXT
- `branch_name`: TEXT
- `is_clean`: INTEGER (Boolean)
- `packet_json`: TEXT (Full ImpactPacket JSON)

### Files Table (Optional for now, but good for future indexing)
- `snapshot_id`: INTEGER
- `path`: TEXT
- `status`: TEXT
- `is_staged`: INTEGER

## Components

### Storage Manager (`src/state/storage.rs`)
- `pub struct StorageManager { conn: Connection }`
- `pub fn init(path: &Path) -> Result<StorageManager>`
- `pub fn save_snapshot(snapshot: &RepoSnapshot, packet: &ImpactPacket) -> Result<()>`
- `pub fn get_latest_snapshot() -> Result<Option<RepoSnapshot>>`

### Migrations (`src/state/migrations.rs`)
- Simple migration runner using `rusqlite_migration`.

## Verification
- Unit tests for database initialization and CRUD operations.
- Integration tests in `tests/persistence.rs` ensuring snapshots are correctly persisted and retrieved across process restarts.

# Track I2-2: Read-Only Storage Init Fast-Path

**Milestone:** I — Issue Remediation  
**Phase:** 2 — Reliability  
**Issue:** CG-6  
**Status:** In Planning

## Objective

Every CLI invocation (including `search`, `ledger status`, `hotspots`, `config verify`) runs the full `StorageManager::init()` path: open SQLite, run migration checks, open CozoDB/Sled, verify schema. This adds 500ms–1s to every command. Read-only commands do not write to storage and do not require migration verification — they only need an open connection.

## Requirements

### `StorageManager::open_read_only(root)`
Add a second constructor that:
1. Opens SQLite in WAL mode (read/write handle — SQLite doesn't support true read-only for WAL, but we skip migration execution).
2. Opens CozoDB/Sled with the existing path (no schema verification).
3. Emits a single `debug!` log (not `info!`) so the init spam disappears from normal output.
4. Does **not** run `run_migrations()` or validate the Cozo schema.

### Read-Only Command Classification
The following commands use `open_read_only`:
- `search`
- `hotspots`
- `ledger status`
- `config verify`
- `doctor` (already does minimal init; apply fast-path here too)

All other commands continue using `StorageManager::init()` (full path).

### Invariant
`open_read_only` must return `Err` (not silently succeed) if the SQLite file does not exist — this distinguishes an uninitialized repo from a read-only access.

## API Contract

```rust
impl StorageManager {
    pub fn init(root: &Utf8Path) -> Result<Self> { ... }           // existing, unchanged
    pub fn open_read_only(root: &Utf8Path) -> Result<Self> { ... } // new
}
```

Both constructors return the same `StorageManager` type. `open_read_only` sets an internal `is_read_only: bool` field. Any attempt to call a write method on a read-only instance should `debug_assert!` (not panic in production).

## Testing Strategy

- Unit test `read_only_skips_migrations`: set up a temp dir with a pre-existing SQLite file (no migrations); call `open_read_only`; assert it succeeds and does not modify the schema version.
- Unit test `read_only_fails_on_missing_db`: temp dir with no SQLite file; call `open_read_only`; assert `Err`.
- Unit test `init_still_runs_migrations`: existing `init` behavior is unchanged.

## Out of Scope

- No connection pooling or shared handles across shell sessions.
- No `--no-storage-check` flag (internal implementation detail, not exposed to CLI).
- CozoDB Sled opening still happens — only migration verification is skipped, not the Sled open itself (which is fast).

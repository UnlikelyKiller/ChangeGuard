# Track I2-2 Plan: Read-Only Storage Init Fast-Path

## Phase 1 — Red (Failing Tests)

- [ ] Add stub `StorageManager::open_read_only` that delegates to `init` (no behavior change yet).
- [ ] Write unit tests:
  - `read_only_skips_migrations`: pre-built SQLite, call `open_read_only`, assert schema version not bumped.
  - `read_only_fails_on_missing_db`: no SQLite file, call `open_read_only`, assert `Err`.
- [ ] Commit: `test(storage): red — open_read_only skips migrations and fails on missing db`

## Phase 2 — Green (Implementation)

- [ ] Add `is_read_only: bool` field to `StorageManager`.
- [ ] Implement `open_read_only`:
  - Return `Err` if SQLite path does not exist.
  - Open SQLite connection (WAL, same as `init` but without `run_migrations()`).
  - Open CozoDB/Sled (same path, no schema validation call).
  - Downgrade all `info!` log calls in this path to `debug!`.
  - Set `is_read_only = true`.
- [ ] In `src/commands/search.rs`, `hotspots.rs`, `ledger/status handler`, `config/verify handler`, `doctor.rs`:
  - Replace `StorageManager::init(root)` with `StorageManager::open_read_only(root)`.
- [ ] Add `debug_assert!(!self.is_read_only, "write called on read-only StorageManager")` to write-path methods.
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `perf(storage): read-only fast-path skips migration check for read commands (CG-6)`

## Verification

- [ ] Time `changeguard search "ledger"` before and after — target <200ms cold start.
- [ ] Confirm no `StorageManager::init called` INFO log on `search` or `ledger status`.
- [ ] `changeguard scan --impact` still uses full init (confirm INFO log present).

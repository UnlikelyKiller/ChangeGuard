# Plan: Track 10 - State SQLite Persistence

### Phase 11: Persistent Ledger
- [ ] Task 10.1: Add `rusqlite` and `rusqlite_migration` as dependencies (already in Cargo.toml).
- [ ] Task 10.2: Implement migration logic in `src/state/migrations.rs`.
  - [ ] Create `snapshots` table.
- [ ] Task 10.3: Implement `StorageManager` in `src/state/storage.rs`.
  - [ ] `init` method with migration execution.
  - [ ] `save_snapshot` method.
  - [ ] `get_latest_snapshot` method.
- [ ] Task 10.4: Integrate `StorageManager` into the `impact` command.
  - [ ] Automatically save every generated packet.
- [ ] Task 10.5: Add unit tests for `StorageManager` in `src/state/storage.rs` using in-memory SQLite.
- [ ] Task 10.6: Add integration tests in `tests/persistence.rs`.
- [ ] Task 10.7: Final verification with `cargo test -j 1 -- --test-threads=1`.

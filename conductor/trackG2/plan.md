## Plan: Track G2 Data Portability & Ledger Integration

### Phase 1: Mirroring SQLite Schema
- [ ] **Task 1.1**: Define the `:create ledger_entry` relation in `cozo.rs` mirroring the SQLite `ledger` table columns.
- [ ] **Task 1.2**: Define the `:create project_symbol` relation in `cozo.rs` mirroring the SQLite `project_symbols` table columns.
- [ ] **Task 1.3**: Define the `:create test_mapping` relation mirroring the SQLite `test_mapping` table.

### Phase 2: In-Process Migration (Porting)
- [ ] **Task 2.1**: Implement `src/state/migration/cozo_port.rs` with a `port_sqlite_to_cozo(sqlite: &SqliteStorage, cozo: &CozoStorage)` function.
- [ ] **Task 2.2**: Use `rusqlite` to stream rows from SQLite and batch-insert into CozoDB via Datalog scripts.
- [ ] **Task 2.3**: Update `src/state/mod.rs` to trigger `port_sqlite_to_cozo` during database initialization.

### Phase 3: Parity Verification (TDD)
- [ ] **Task 3.1**: Write a test that inserts 100 ledger entries into SQLite and verifies they are correctly ported to CozoDB.
- [ ] **Task 3.2**: Write a test verifying that symbol queries (`find_symbol_by_name`) return identical results from both backends during the transition.
- [ ] **Task 3.3**: Verify data integrity for complex JSON metadata stored in both databases.

### Definition of Done (DoD)
- [x] Existing project data is successfully migrated to CozoDB.
- [x] Symbol lookup parity is 100%.
- [x] Migration logic handles empty SQLite databases gracefully.
- [x] No more than 4 files modified: `src/state/storage/cozo.rs`, `src/state/mod.rs`, `src/state/migration/mod.rs`, `src/state/migration/cozo_port.rs`.

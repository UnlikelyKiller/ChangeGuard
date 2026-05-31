# Track RE6: Standardize `src/state/storage_cozo.rs`

## Objective
Decouple the CozoDB Datalog queries and schema management from the core storage manager.

## Requirements
- **Datalog Management**: Move all raw Datalog query strings to a dedicated `src/state/cozo/queries.rs`.
- **Migration Logic**: Move schema migration and CozoDB initialization to `src/state/cozo/init.rs`.
- **Abstraction**: Ensure `CozoStorage` only contains the high-level API methods, delegating the low-level string manipulation.

## Definition of Done (DoD)
- [ ] `src/state/storage_cozo.rs` is reduced to < 400 lines.
- [ ] Queries are categorized and documented in `queries.rs`.
- [ ] Schema management is isolated from data operations.
- [ ] All 897 tests pass, especially `tests/cozo_schema_migration.rs` and `tests/cozodb_integrity.rs`.

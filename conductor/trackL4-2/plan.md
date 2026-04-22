## Plan: Track L4-2 FTS5 Search Integration
### Phase 1: Database Operations
## Plan: L4-2 FTS5 Search Integration
### Phase 1: Database Operations
- [x] Task 1.1: Update `src/ledger/db.rs` to implement the `search_ledger` method querying the `ledger_fts` table, joining with `ledger_entries`.
- [x] Task 1.2: Add logic to conditionally append SQL clauses for `category`, `days`, and `breaking_only` filters.
- [x] Task 1.3: Ensure results are ordered by `f.rank` and `l.committed_at DESC`, and handle FTS5 syntax errors gracefully to provide clear error messages.
### Phase 2: CLI Integration
- [x] Task 2.1: Add `Search` variant to `LedgerCommands` enum in `src/cli.rs`.
- [x] Task 2.2: Add routing logic in `src/cli.rs` for `LedgerCommands::Search`.
- [x] Task 2.3: Create `src/commands/ledger_search.rs` and implement `execute_ledger_search` to execute the query and format results nicely for the terminal.
- [x] Task 2.4: Export `ledger_search` in `src/commands/mod.rs`.
### Phase 3: Testing
- [x] Task 3.1: Create `tests/ledger_search.rs`.
- [x] Task 3.2: Write tests for full-text search across `entity`, `summary`, and `reason`.
- [x] Task 3.3: Write integration tests verifying the `category`, `days`, and `breaking_only` filters limit the results correctly.
- [x] Task 3.4: Write tests verifying the deterministic ranking order of search results.
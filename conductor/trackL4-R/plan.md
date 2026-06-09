## Plan: Track L4-R: Search & ADR Remediation
### Phase 1: Core Database & Template Fixes
- [ ] Task 1.1: Fix FTS alias in `search_ledger` query in `src/ledger/db.rs` (change `ledger_fts MATCH` to `f MATCH`).
- [ ] Task 1.2: Update `--days` filtering in `src/ledger/db.rs` for both `get_adr_entries` and `search_ledger` to use `strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-{} days')` instead of `datetime()`.
- [ ] Task 1.3: Update error handling in `search_ledger` inside `src/ledger/db.rs` to map `rusqlite::Error` to `LedgerError::Validation` during row iteration, specifically looking for "syntax error".
- [ ] Task 1.4: Update `generate_madr_content` in `src/ledger/adr.rs` to include `## Decision` and `## Consequences` sections consistently. Update `test_generate_madr_content` to assert these section inclusions.

### Phase 2: TransactionManager Integration
- [ ] Task 2.1: Add `search_ledger` wrapper method to `TransactionManager` in `src/ledger/transaction.rs`.
- [ ] Task 2.2: Refactor `execute_ledger_search` in `src/commands/ledger_search.rs` to instantiate `TransactionManager` with `get_repo_root()` and use its `search_ledger` method.

### Phase 3: Testing
- [ ] Task 3.1: Update `tests/ledger_search.rs::test_search_invalid_syntax` to assert that the returned error is explicitly the mapped `Validation` error type rather than a generic error.
- [ ] Task 3.2: Add a new test in `tests/ledger_search.rs` for `--days` filtering to verify correct RFC3339 date boundaries and comparison.
- [ ] Task 3.3: Verify that existing tests in `tests/ledger_search.rs` cover the alias fix (by running basic searches) and that no further panics occur.
# Track L4-R: Search & ADR Remediation Specification

## 1. Objective
Remediate findings from the Phase L4 Codex review to ensure FTS search correctness, reliable datetime filtering, robust error handling, standard MADR compliance, and consistent use of `TransactionManager` for all ledger queries.

## 2. Findings & Remediations

### 2.1 FTS Alias Mismatch
**Issue:** `ledger search` query uses `WHERE ledger_fts MATCH ?1` while the table is aliased as `f`. This leads to SQLite name resolution failures and search malfunction.
**Fix:** Update `search_ledger` in `src/ledger/db.rs` to use `WHERE f MATCH ?1` or consistently avoid aliasing the FTS table (e.g., using `ledger_fts MATCH` and `ledger_fts.rank`). Using `f MATCH ?1` is standard for SQLite FTS5 table aliases.

### 2.2 Timestamp Format Filtering
**Issue:** `--days` filtering in `get_adr_entries` and `search_ledger` compares stored RFC3339 values (e.g. `2026-04-22T10:00:00Z`) against `datetime('now', '-N days')` which produces an incompatible format (e.g. `2026-04-22 10:00:00`). This leads to incorrect lexicographic sorting.
**Fix:** Update both queries in `src/ledger/db.rs` to use `strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-N days')` to produce properly formatted RFC3339 strings for correct comparison.

### 2.3 Search Error Handling
**Issue:** FTS syntax errors are only mapped during `prepare()`. However, FTS5 `MATCH` parse errors usually occur when iterating over rows (`query_map` or fetching).
**Fix:** Update `search_ledger` in `src/ledger/db.rs` to catch SQLite failures inside the row iteration loop (when unpacking the `Result` from the mapped rows) and map them to `LedgerError::Validation`. Ensure `test_search_invalid_syntax` in `tests/ledger_search.rs` asserts on the specific `LedgerError::Validation` error variant and message.

### 2.4 MADR Template Compliance
**Issue:** The exported MADR files lack a `## Decision` section and conditionally omit the `## Consequences` section unless `outcome_notes` exists.
**Fix:** In `src/ledger/adr.rs`, update `generate_madr_content` to always output:
- `## Decision` (containing the entry summary).
- `## Consequences` (containing the `outcome_notes` if available. If `outcome_notes` is `None` and `is_breaking` is true, output a standard breaking change warning. Otherwise, output "None."). Update tests to verify.

### 2.5 TransactionManager Wrapper
**Issue:** The `ledger search` command currently bypasses `TransactionManager` and calls `LedgerDb` directly.
**Fix:** Add `search_ledger` to `TransactionManager` in `src/ledger/transaction.rs`. Update `execute_ledger_search` in `src/commands/ledger_search.rs` to initialize `TransactionManager` and use its `search_ledger` method.

## 3. Deliverables
- `src/ledger/db.rs`: Fix FTS alias, update `strftime` for days filtering, and catch row-level FTS syntax errors.
- `src/ledger/adr.rs`: Update MADR template structure to include Decision and Consequences.
- `src/ledger/transaction.rs`: Add `search_ledger` wrapper function.
- `src/commands/ledger_search.rs`: Refactor to use `TransactionManager`.
- `tests/ledger_search.rs`: Enhance tests to cover days filtering logic, robust FTS syntax error mapping, and ensure the alias mismatch is definitively fixed.
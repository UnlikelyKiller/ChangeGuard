# Track X6 Plan: `audit` File Path Resolution

## Phase 1 — Red (Failing Tests)
- [ ] 1. Write integration test: create a ledger transaction that touched `src/commands/audit.rs`, run `changeguard ledger audit src/commands/audit.rs`, assert the transaction appears in output.
- [ ] 2. Write unit test for `looks_like_file_path(s: &str) -> bool` helper: assert true for `src/foo.rs`, `C:\dev\foo.rs`, false for `my-service`.

## Phase 2 — Implementation
- [ ] 3. Add `fn looks_like_file_path(s: &str) -> bool` in `src/commands/ledger_audit.rs`:
  - Returns `true` if `s` contains `/` or `\` or has an extension (`.` before last component).
- [ ] 4. In `execute_ledger_audit`, after entity-name lookup:
  - If `looks_like_file_path(&entity)`, normalize to canonical form (try `dunce::canonicalize`, fall back to as-is).
  - Query `project_file_changes` for transactions matching the file path (exact match + suffix match).
  - Merge results with entity-name results.
- [ ] 5. Add `LedgerDb::find_transactions_by_file(file_path: &str) -> Result<Vec<Transaction>>` in `src/ledger/db.rs`:
  ```sql
  SELECT DISTINCT t.* FROM ledger_transactions t
  JOIN project_file_changes pfc ON t.id = pfc.transaction_id
  WHERE pfc.file_path = ?1 OR pfc.file_path LIKE '%' || ?1
  ORDER BY t.created_at DESC
  ```
- [ ] 6. Print a note when file-path results are included:
  ```rust
  println!("  {}", format!("Showing transactions that touched file: {}", entity).dimmed());
  ```

## Phase 3 — Green + Cleanup
- [ ] 7. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 8. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 9. Run `cargo fmt --all -- --check` — clean.
- [ ] 10. Update `conductor/conductor.md` status to Completed.

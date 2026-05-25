# Track Q4: Ledger API Consistency & ADR UX

## Objective
1. **Auditable Rollbacks**: Add a `--reason` flag to the `ledger rollback` command so that every state change (including rollbacks) carries explicit intent.
2. **ADR UX Guidance**: Provide helpful usage hints when `ledger adr` returns no results, explaining how ADRs are generated from Architecture and Breaking changes.

## Requirements
1. **Rollback Reason**:
   - The `changeguard ledger rollback` command must require a `--reason` flag.
   - The `TransactionManager::rollback_change` function must accept this reason.
   - To make it auditable, the rollback should ideally insert a `LedgerEntry` with the rollback reason (introducing `EntryType::Rollback` if appropriate) OR the `transactions` table must track this reason (by updating the `reason` column in `transactions`).
   - Given the codebase style, `TransactionManager::rollback_change` should insert a `LedgerEntry` (with `EntryType::Rollback`) to provide a fully verifiable, signed audit log, matching how `reconcile_drift` and `commit_change` work.
2. **ADR Empty State UX**:
   - In `execute_ledger_adr`, when `entries.is_empty()` evaluates to true, print a helpful, multi-line message explaining what constitutes an ADR.
   - Include an example command showing how to create an ADR (e.g., using `--category architecture` and `--breaking`).

## Testing Strategy
1. **Compilation Check**: Verify `cargo check` and `cargo test` pass.
2. **Rollback CLI Test**: Start a transaction and rollback using `changeguard ledger rollback --reason "test reason"`. Verify it succeeds and is tracked.
3. **ADR UX Test**: Run `changeguard ledger adr` on a fresh repository (or one with no ADR entries) and verify the explanatory message is printed.

## Plan: Ledger API Consistency & ADR UX
### Phase 1: Support Auditable Rollbacks
- [x] Task 1.1: Add `Rollback` variant to `EntryType` enum in `src/ledger/types.rs`.
- [x] Task 1.2: Add `#[arg(short, long)] reason: String` to `LedgerCommands::Rollback` in `src/cli.rs`.
- [x] Task 1.3: Update `execute_ledger_rollback` signature in `src/commands/ledger.rs` to accept `reason: String` and pass it to `tx_mgr.rollback_change()`.
- [x] Task 1.4: Update `TransactionManager::rollback_change` in `src/ledger/transaction.rs` to accept `reason: String`.
- [x] Task 1.5: Inside `rollback_change`, create and insert a `LedgerEntry` (with `EntryType::Rollback`, signed via `sign_ledger_entry` if signing is active) before updating the transaction status to `ROLLED_BACK`.
- [x] Task 1.6: Update `TransactionManager::atomic_change` to pass a fallback reason (e.g., `"Rollback after commit failure"`) when calling `rollback_change` during a failure.

### Phase 2: Improve ADR Empty State UX
- [x] Task 2.1: Locate the `entries.is_empty()` check in `src/commands/ledger_adr.rs`.
- [x] Task 2.2: Add informative `println!` output explaining how ADRs are generated (Category == Architecture or `--breaking` is used).
- [x] Task 2.3: Include an example command (e.g., `changeguard ledger start src/arch/new_design.md -c architecture` and `changeguard ledger commit --summary "Use event sourcing" --reason "High throughput needs"`) in the empty state output.

### Phase 3: Final Verification
- [x] Task 3.1: Run `cargo check` and `cargo test` to ensure successful compilation and no regressions.
- [x] Task 3.2: Verify manually that `changeguard ledger adr` prints the correct guidance when no ADRs exist.
- [x] Task 3.3: Verify manually that `changeguard ledger rollback --reason "test"` executes successfully.
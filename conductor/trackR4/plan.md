## Plan: Ledger Lifecycle Hardening (GC & Orphans)
### Phase 1: CLI & Args
- [x] Task 1.1: Add `gc` subcommand with an `--orphans` flag to `src/cli.rs`.
### Phase 2: State Pruning
- [x] Task 2.1: Implement queries in `src/ledger/db.rs` to find stale/orphaned `PENDING` transactions.
- [x] Task 2.2: Implement deletion/pruning logic safely in `execute_ledger_gc`.
### Phase 3: Verification
- [x] Task 3.1: Create a dummy pending transaction, wait (or mock time), and run `ledger gc --orphans`.
- [x] Task 3.2: Verify the transaction is removed from `ledger status`.
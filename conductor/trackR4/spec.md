# Specification: Ledger Lifecycle Hardening (GC & Orphans)

## Objective
Implement garbage collection for the ChangeGuard ledger to remove orphaned or stale `PENDING` transactions, preventing state bloat and confusing provenance reports.

## Requirements
- Target files: `src/commands/ledger.rs`, `src/ledger/transaction.rs` or `src/state/cozo/queries.rs`.
- Add subcommand `changeguard ledger gc --orphans`.
- Identify transactions that are:
  - Marked `PENDING`.
  - Older than a reasonable TTL (e.g., 7 days) OR where the associated branch/commit no longer exists.
- Remove these entries from the CozoDB ledger state.

## Architecture
- Add `LedgerGcArgs` struct to `src/commands/ledger.rs`.
- Implement `delete_orphaned_transactions` in the ledger DB layer.
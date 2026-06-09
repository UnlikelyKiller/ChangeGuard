# Specification: Track L-H1 - Ledger Production Hardening (v2)

## Overview
Track L-H1 addresses critical and high-severity findings for Milestone L, as well as discoverability issues for key commands. While partially implemented, gaps remain in bulk updates, path normalization usage, reset command validation, and federation root discovery. Additionally, the `audit` command must be exposed at the top level for discoverability.

## Detailed Requirements

### 1. Lifecycle Invariants (High Severity)
**Problem:** Concurrent modifications during bulk updates can lead to invalid state transitions. `update_transaction_status_bulk` lacks state validation.
**Solution:**
- Update `update_transaction_status_bulk` in `src/ledger/db.rs` to accept an `expected_status` parameter and append `AND status = ?` to the `WHERE` clause.
- Modify the return type to `Result<usize, LedgerError>` to return the number of affected rows.
- Ensure callers (like `adopt_drift` and `reconcile_drift` in `src/ledger/transaction.rs`) pass the correct expected status (`UNAUDITED`). If the number of affected rows does not match the number of IDs passed, log a warning or handle the concurrency failure.

### 2. Durable State Protection & Path Normalization (High/Medium Severity)
**Problem:** Verification is needed to guarantee that `ledger.db` is protected during `reset` and that `normalize_relative_path` is exclusively used for path resolution.
**Solution:**
- **Reset Command Verification:** Ensure `src/commands/reset.rs` properly respects the `--include-ledger` flag and write/verify tests to assert this behavior.
- **Path Normalization:** Audit and strictly enforce `normalize_relative_path` usage in `src/ledger/transaction.rs`, `src/ledger/drift.rs`, and `src/ledger/federation.rs`. Verify no ad-hoc fallback logic bypasses this mechanism.

### 3. Federation Root Discovery (Medium Severity)
**Problem:** Ensure federate commands discover the git root correctly, preventing errors when run from subdirectories.
**Solution:**
- Verify `execute_federate_export`, `execute_federate_scan`, and `execute_federate_status` in `src/commands/federate.rs` properly resolve the Git repository root using `open_repo` instead of blindly trusting `env::current_dir()`. 
- Add tests to confirm federate commands work correctly from nested subdirectories.

### 4. Command Discoverability (UX)
**Problem:** `changeguard audit` returns an unrecognized subcommand error because it is nested under `changeguard ledger audit`.
**Solution:**
- Add `Audit` as a top-level subcommand in `src/cli.rs`.
- Wire the top-level `Audit` command to call `crate::commands::ledger_audit::execute_ledger_audit` (the same underlying logic as `ledger audit`).
- Keep `ledger audit` for backward compatibility, ensuring both paths route to the same handler.

## FFI/API Contracts
- `src/ledger/db.rs::update_transaction_status_bulk(tx_ids: &[String], status: &str, expected_status: &str, resolved_at: Option<&str>) -> Result<usize, LedgerError>`
  - Must return the number of rows updated or an error. Must accept an `expected_status` argument to ensure safe state transitions.
- `src/cli.rs::Commands`
  - Must include `Audit { entity: Option<String>, include_unaudited: bool }` mimicking the `LedgerCommands::Audit` structure.
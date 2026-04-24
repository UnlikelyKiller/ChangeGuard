## Plan: Ledger Production Hardening & UX Fixes (Track L-H1 v2)

### Phase 1: Ledger Lifecycle & Bulk Updates
- [x] Task 1.1: Modify `src/ledger/db.rs` `update_transaction_status_bulk` to accept an `expected_status: &str` argument and return `Result<usize, LedgerError>`.
- [x] Task 1.2: Update the SQL query in `update_transaction_status_bulk` to include `AND status = ?` using the `expected_status`.
- [x] Task 1.3: Update callers in `src/ledger/transaction.rs` (`reconcile_drift` and `adopt_drift`) to pass `"UNAUDITED"` as the expected status and verify the affected row count.
- [x] Task 1.4: Add unit tests to verify bulk update concurrency protection.

### Phase 2: Path Normalization & Reset Verification
- [x] Task 2.1: Audit `src/ledger/transaction.rs`, `src/ledger/drift.rs`, and `src/ledger/federation.rs` to ensure `normalize_relative_path` is exclusively used. Refactor any remaining ad-hoc path logic.
- [x] Task 2.2: Add integration tests in `tests/cli_reset.rs` to explicitly verify `ledger.db` is preserved during default reset and deleted when `--include-ledger` is provided.

### Phase 3: Federation Root Discovery
- [x] Task 3.1: Audit `src/commands/federate.rs` (Export, Scan, Status) to ensure Git repo root is resolved via `open_repo` for layout construction, rather than raw `env::current_dir()`.
- [x] Task 3.2: Add an integration test in `tests/federated_discovery.rs` to run a federate command from a subdirectory and ensure it targets the correct root `.changeguard/state/ledger.db`.

### Phase 4: Audit Command Discoverability
- [x] Task 4.1: Update `src/cli.rs` to add the `Audit` variant to the top-level `Commands` enum, including `--entity` and `--include-unaudited` arguments.
- [x] Task 4.2: Update `run()` in `src/cli.rs` to map the top-level `Commands::Audit` to `crate::commands::ledger_audit::execute_ledger_audit`.
- [x] Task 4.3: Add a test in `tests/cli_verify.rs` or `tests/cli_binary.rs` to ensure `changeguard audit` parses correctly and executes.

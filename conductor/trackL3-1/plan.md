## Plan: Ledger-Aware Scan & Impact

### Phase 1: Data Model & Ledger Retrieval Foundation
- [ ] Task 1.1: Update `src/impact/packet.rs`. Add `tx_id`, `category`, and `planned_action` (all `Option<String>`) to the `ChangedFile` struct. Add `#[serde(default, skip_serializing_if = "Option::is_none")]` annotations for backward compatibility. Update tests.
- [ ] Task 1.2: Update `src/ledger/db.rs`. Implement `get_pending_by_entities(entities: &[String]) -> Result<Vec<Transaction>, LedgerError>` to efficiently bulk retrieve PENDING transactions.
- [ ] Task 1.3: Update `src/ledger/transaction.rs`. Add `get_pending_for_paths(paths: &[impl AsRef<Path>]) -> Result<HashMap<PathBuf, Transaction>, LedgerError>` to `TransactionManager` which normalizes paths and calls the new DB method.

### Phase 2: Impact Command Integration
- [ ] Task 2.1: Update `src/commands/impact.rs`. In `execute_impact`, initialize SQLite (`StorageManager`) and `TransactionManager` early. Use the `get_pending_for_paths` method on all modified file paths from the snapshot.
- [ ] Task 2.2: Update `map_snapshot_to_packet` in `src/commands/impact.rs` to accept the transaction map and populate the new `tx_id`, `category`, and `planned_action` fields for each `ChangedFile` entry. Handle the missing ledger database gracefully.
- [ ] Task 2.3: Update `tests/cli_impact.rs`. Add an integration test that creates a ledger transaction and verifies the `tx_id` is present in the generated `impact` JSON report.

### Phase 3: Scan Command Integration
- [ ] Task 3.1: Update `src/output/human.rs`. Modify `print_scan_summary` to accept `tx_map: Option<&HashMap<PathBuf, String>>`. Update the CLI table definition to `["State", "Action", "Tx", "File Path"]`. Display the truncated `tx_id` (first 8 characters) or "UNTRACKED" (dimmed).
- [ ] Task 3.2: Update `src/commands/scan.rs`. In `execute_scan`, attempt to query the `StorageManager` and `TransactionManager` for all changed paths to construct the `tx_map`.
- [ ] Task 3.3: Pass the mapping into `print_scan_summary`. Ensure that if the database is missing or an error occurs, it falls back to an empty map (displaying "UNTRACKED").
- [ ] Task 3.4: Update `tests/cli_scan.rs`. Add an integration test where a pending transaction exists, verifying the CLI table output contains the `tx_id`.

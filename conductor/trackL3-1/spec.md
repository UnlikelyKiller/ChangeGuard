# Specification: Track L3-1: Ledger-Aware Scan & Impact

## Overview
This track implements the foundational layer of Phase L3 / L5 (Change Intelligence Integration) by bridging the gap between ChangeGuard's scan/impact tools and the Ledger's provenance tracking. It involves augmenting the CLI output of `scan` to reflect active (PENDING) transactions, embedding transaction metadata into the `ImpactPacket`, and providing bulk ledger retrieval utilities.

## Requirements

### 1. `src/ledger/transaction.rs` Helper Methods
- **`get_pending_for_paths`**: Add a method to `TransactionManager` that accepts a list of file paths (as strings or `Path`s) and bulk-queries the `LedgerDb` for PENDING transactions associated with those normalized paths.
- Returns a mapping (e.g., `HashMap<String, Transaction>`) of normalized path to the associated transaction.
- Implementation should map paths using `entity_normalized` before querying. A new bulk query method in `LedgerDb` (e.g., `get_pending_by_entities`) should be implemented using an `IN` clause or by iterating.

### 2. `src/impact/packet.rs` Data Model Updates
- Update the `ChangedFile` struct to include:
  - `pub tx_id: Option<String>`
  - `pub category: Option<String>`
  - `pub planned_action: Option<String>`
- Use `#[serde(default, skip_serializing_if = "Option::is_none")]` annotations for backward compatibility with existing `.json` packets.

### 3. `src/commands/impact.rs` Updates
- During `execute_impact`, attempt to initialize the `StorageManager`.
- If successful, use a `TransactionManager` to query PENDING transactions for all detected changed files.
- Inject the `tx_id`, `category`, and `planned_action` into each `ChangedFile` entry in the `ImpactPacket`.
- This ensures the JSON impact report written to `.changeguard/reports/` includes transaction metadata for downstream processors.

### 4. `src/commands/scan.rs` Updates
- Initialize `StorageManager` during `execute_scan`.
- Query PENDING transactions for all files in the `RepoSnapshot`.
- Update `src/output/human.rs` -> `print_scan_summary` to accept the transaction mapping (e.g., `Option<&HashMap<PathBuf, String>>`).
- Modify the scan summary table to include a new "Tx" column (`["State", "Action", "Tx", "File Path"]`).
- For each path, display the truncated `tx_id` (e.g., first 8 characters) or "UNTRACKED" (dimmed) if no transaction exists.

## Technical Details & Constraints
- **Graceful Degradation**: If `StorageManager` fails to initialize (e.g., outside a changeguard context), `scan` and `impact` should fall back to standard behavior with `None` or "UNTRACKED".
- **Path Normalization**: The `entity_normalized` logic from `TransactionManager` must be used strictly to match git status paths with database entities.
- **TDD Requirement**: Integration tests must be added to `tests/cli_scan.rs` and `tests/cli_impact.rs` validating behavior with and without active ledger transactions.

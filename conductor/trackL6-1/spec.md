# Specification: Track L6-1: Ledger Federation

## 1. Objective
Implement cross-repo ledger federation, as defined in Phase L6 of `docs/Ledger-Incorp-plan.md`. This feature allows exporting local ledger entries to a `schema.json` file and importing entries from sibling repositories, storing them with a `SIBLING` origin. The `impact` command will be updated to show these federated ledger entries when a local repo depends on a modified sibling entity.

## 2. Deliverables & Technical Requirements

### 2.1 Schema Updates (`src/federated/schema.rs`)
- Extend `FederatedSchema` to include an optional `ledger` field:
  ```rust
  pub ledger: Option<Vec<FederatedLedgerEntry>>
  ```
- Define `FederatedLedgerEntry` containing: `tx_id`, `category`, `entry_type`, `entity`, `change_type`, `summary`, `reason`, `is_breaking`, `committed_at`.
- Validation: Ensure `entity` paths do not contain path traversal elements (`..`).

### 2.2 Federation Logic (`src/ledger/federation.rs` - NEW)
- **Exporting (`export_ledger_entries`)**:
  - Query local `ledger_entries` where `origin = 'LOCAL'` and `committed_at` is within the last 30 days (to limit `schema.json` size).
  - Map results to `FederatedLedgerEntry`.
- **Importing (`import_federated_entries`)**:
  - Accept `sibling_name` and a slice of `FederatedLedgerEntry`.
  - For each entry, enforce path confinement (reject `..` or absolute paths).
  - To satisfy foreign key constraints on `transactions(tx_id)`, insert a stub/dummy transaction if one doesn't exist: `status='COMMITTED'`, `source='SIBLING'`, `session_id='federated'`.
  - Insert into `ledger_entries` with `origin='SIBLING'` and `trace_id=sibling_name` (using `INSERT OR IGNORE` or explicit conflict handling).

### 2.3 CLI Integration (`src/commands/federate.rs`)
- **`execute_federate_export`**: Call `export_ledger_entries` and populate the `ledger` array in `FederatedSchema` before serialization.
- **`execute_federate_scan`**: When processing a sibling schema, if `schema.ledger` is present, call `import_federated_entries`.

### 2.4 Transaction Manager & Query Updates
- **`src/ledger/search.rs` / Query Logic**: Update queries to return `origin` and `trace_id`.
- **UX Marking**: When printing ledger entries (e.g., in `ledger search` or `ledger audit`), if `origin == "SIBLING"`, prepend `[FEDERATED]` to the display (e.g., `[FEDERATED] <tx_id> - <summary> (from <trace_id>)`).

### 2.5 Cross-Repo Impact (`src/federated/impact.rs`)
- During cross-repo impact analysis, for each `(local_symbol, sibling_symbol)` dependency:
  - Identify the `sibling_file` from the sibling's `schema.public_interfaces`.
  - Query the local DB for federated ledger entries (`origin='SIBLING'`, `trace_id=sibling_name`, `entity_normalized=sibling_file`) from the last 7/30 days.
  - Append findings to `impact_reasons`: `Cross-repo impact: Sibling '<name>' modified '<file>' ([FEDERATED] <summary>)`.

## 3. Testing Strategy (TDD)
- **Unit Tests**: Add `tests/federated_ledger.rs` (or extend existing tests) testing the export and import logic with a mock SQLite database.
- **Path Confinement Test**: Verify that importing an entry with `entity: "../../../etc/passwd"` is rejected.
- **Impact Integration Test**: Mock a `schema.json` with a ledger entry, run `federate scan`, and verify `impact` output includes the federated entry warning.

# Specification: Track L2-2 - Ledger Reconciliation & Adoption

## Objective
Implement Phase L2 (Drift Detection & Reconciliation) completion as defined in `docs/Ledger-Incorp-plan.md`. This enables users to resolve untracked filesystem changes (drift) by either reconciling them retroactively or adopting them into a new pending transaction.

## Functional Requirements

### 1. Reconcile (`UNAUDITED` -> `RECONCILED`)
When a file watcher detects a change without a PENDING transaction, it creates an `UNAUDITED` entry. The `ledger reconcile` command acknowledges this drift.
- **State Transition**: `UNAUDITED` -> `RECONCILED`.
- **Provenance**: Must preserve `source: WATCHER`.
- **Ledger Entry**: A formal entry is written to `ledger_entries`, securing the immutable audit trail.
- **Aggregation / Bulk**: Support reconciling multiple UNAUDITED records via `--entity-pattern` (glob).
- **Auto-Reconcile**: Automatically reconcile any UNAUDITED entries for an entity when a `ledger commit` happens for that same entity (if `--auto-reconcile` is configured or provided).

### 2. Adopt (`UNAUDITED` -> `PENDING`)
Users can choose to actively work on a drifted file. `ledger adopt` brings it into the standard workflow.
- **State Transition**: `UNAUDITED` -> `PENDING`.
- **Reasoning**: A reason must be provided (added to transaction notes/description).

### 3. Status View Updates
- `ledger status` should correctly categorize and display `UNAUDITED` vs `PENDING` vs `RECONCILED` entries, differentiating active session work from stale drift.

## System Architecture

### 1. `src/ledger/transaction.rs`
Add logic to `TransactionManager`:
- `reconcile_unaudited(tx_id_or_prefix: &str, req: CommitRequest) -> Result<(), LedgerError>`
  1. Resolve `tx_id`.
  2. Verify transaction is `UNAUDITED`.
  3. Update status to `RECONCILED` with `resolved_at` set.
  4. Create `LedgerEntry` (with `source: WATCHER` passed from the transaction).
- `adopt_unaudited(tx_id_or_prefix: &str, reason: String) -> Result<(), LedgerError>`
  1. Resolve `tx_id`.
  2. Verify transaction is `UNAUDITED`.
  3. Update status to `PENDING`, clearing `detected_at` / `drift_count` logically (or storing provenance).
- `auto_reconcile_entity(entity_normalized: &str, req: CommitRequest) -> Result<usize, LedgerError>`
  1. Find all `UNAUDITED` transactions for the entity.
  2. Map them through `reconcile_unaudited` silently.

### 2. `src/ledger/db.rs`
- Add `get_unaudited_by_pattern(pattern: &str) -> Result<Vec<Transaction>, LedgerError>` to support `--entity-pattern`.
- Ensure querying by `status = 'UNAUDITED'` and `entity_normalized = ?` is efficient.

### 3. `src/commands/ledger.rs` (or dedicated submodules)
- `execute_ledger_reconcile(tx_id: Option<String>, entity_pattern: Option<String>, summary: String, reason: String, auto_reconcile: bool)`
  - Handle singular vs bulk reconciliation.
- `execute_ledger_adopt(tx_id: String, reason: String)`
- Update `execute_ledger_commit` to invoke `tx_mgr.auto_reconcile_entity(...)` if `auto_reconcile` is active.

### 4. Tests (`tests/ledger_drift.rs`)
- Test: `UNAUDITED` creation on untracked change.
- Test: Reconciliation round trip (`UNAUDITED` -> `RECONCILED`).
- Test: Bulk reconciliation by pattern.
- Test: Deduplication of same-entity drift.
- Test: Auto-reconcile integration at commit time.
- Test: Adopt round trip (`UNAUDITED` -> `PENDING`).

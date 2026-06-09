# Specification: Track L6-R: Ledger Federation Remediation

## 1. Objective
Address the high and medium severity findings from the Codex review for Track L6-1 to ensure correct federation identity, security, export limits, schema alignment, impact integration, audit UX, and test alignment.

## 2. Deliverables & Technical Requirements

### 2.1 Schema Alignment (`src/federated/schema.rs`)
- Remove `origin` from `FederatedLedgerEntry` to comply with the specified schema shape.

### 2.2 Federation Logic Updates (`src/ledger/federation.rs`)
- **Federation Identity (Import)**: Store `origin = 'SIBLING'` and `trace_id = sibling_name` for imported entries. Update duplicate checks to key on `origin = 'SIBLING'` and `trace_id = sibling_name`.
- **Security (Import)**: Enforce strict path confinement in `import_federated_entries`. Reject paths containing `..`, absolute paths, Windows rooted paths, and UNC paths.
- **Export Limit (Export)**: Implement the 30-day limit in `export_ledger_entries`. Use RFC3339 comparison against the `committed_at` field to filter entries to the last 30 days.
- **Export/Import Mapping**: Adjust mapping logic to reflect the removal of `origin` from `FederatedLedgerEntry`.

### 2.3 Impact Integration (`src/federated/impact.rs`)
- Refactor the cross-repo impact analysis to query the local ledger DB for imported rows where `origin = 'SIBLING'`, `trace_id = sibling_name`, and `entity_normalized = sibling_file`.
- Remove direct reading from `schema.ledger`.
- Include recent time window filtering (e.g., last 30 days) if applicable.

### 2.4 CLI Integrations and UX Updates
- **Audit UX (`src/commands/ledger_audit.rs`)**: Update output to check `origin == "SIBLING"` and display `[FEDERATED: {trace_id}]`.

### 2.5 Test Alignment (`tests/ledger_federation.rs`)
- Fix the function signature mismatch. Update calls to `execute_scan` to match the current API: `execute_scan(run_impact: bool)`.

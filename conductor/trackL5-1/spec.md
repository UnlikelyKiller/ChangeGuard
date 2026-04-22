# Specification: Track L5-1: Token-Level Provenance

## Objective
Implement token-level attribution to transactions, as defined in Phase L5. This enables ChangeGuard to record exactly which symbols (functions, structs, classes, etc.) were modified, added, or deleted in each transaction, bringing architectural memory down to the token level.

## Deliverables

1. **`src/ledger/provenance.rs`**
   - Core logic, defining `TokenProvenance`, `ProvenanceAction` (`Added`, `Modified`, `Deleted`), and logic for comparing sets of symbols.

2. **`src/state/migrations.rs` Update**
   - Add Migration M14 (or next available) to create the `token_provenance` table.

3. **`TransactionManager` Updates**
   - Methods to record and retrieve symbol changes.

4. **CLI Commands (`src/commands/`)**
   - `ledger track`: New command to manually attach token provenance to a pending transaction.
   - `ledger commit`: Integrates token-level provenance.
   - `ledger audit`: Updates the output of `--entity` to show a timeline of symbol modifications.
   - `ledger status`: Displays symbol summaries for transactions.

## Data Model (SQLite)

Add the following table in a new migration:

```sql
CREATE TABLE IF NOT EXISTS token_provenance (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_id              TEXT NOT NULL REFERENCES transactions(tx_id),
    entity             TEXT NOT NULL,
    entity_normalized  TEXT NOT NULL,
    symbol_name        TEXT NOT NULL,
    symbol_type        TEXT NOT NULL,
    action             TEXT NOT NULL -- 'ADDED', 'MODIFIED', 'DELETED'
);

CREATE INDEX IF NOT EXISTS idx_token_provenance_tx_id ON token_provenance(tx_id);
CREATE INDEX IF NOT EXISTS idx_token_provenance_entity_symbol ON token_provenance(entity_normalized, symbol_name);
```

## Logic & Integration (`src/ledger/provenance.rs`)

We will define structures and operations to interact with `token_provenance`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceAction {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct TokenProvenance {
    pub tx_id: String,
    pub entity: String,
    pub symbol_name: String,
    pub symbol_kind: String, // Maps to SymbolKind
    pub action: ProvenanceAction,
}
```

The system will leverage existing ChangeGuard symbol extraction (`src/index/symbols.rs`) to detect symbols. A utility function `compute_symbol_diff(old_symbols, new_symbols)` will identify changes.

## CLI Commands

### 1. `ledger track`
```bash
changeguard ledger track --tx-id <UUID> --entity src/main.rs --symbol "run_app" --symbol-type "Function" --action MODIFIED
```
Explicitly attaches a symbol modification to a transaction (useful for fine-grained manual overrides or AI agents).

### 2. `ledger commit`
Optionally accept `--track-symbol` arguments or automatically compute token provenance by extracting symbols from the current entity file and comparing against the last snapshot.

### 3. `ledger audit`
When running `changeguard ledger audit --entity <path>`, the history timeline should interleave or attach `token_provenance` records, so users can see precisely *when* and *why* a specific function or struct was altered.

## Acceptance Criteria
- Integration tests in `tests/ledger_provenance.rs` follow TDD.
- `ledger track` inserts rows properly.
- `ledger commit` properly handles and finalizes token-level constraints.
- `ledger audit --entity` includes `[Function: run_app - MODIFIED]` markers.

# Track L4-2: FTS5 Search Integration Specification

## 1. Objective
Implement the `ledger search` command to perform full-text search across ledger entries using the SQLite FTS5 virtual table (`ledger_fts`), as defined in Phase L4 of `docs/Ledger-Incorp-plan.md`.

## 2. Deliverables
- `src/commands/ledger_search.rs`: Implementation of the CLI command logic.
- `src/ledger/db.rs` updates: Addition of `search_ledger` method.
- `src/cli.rs`: Registration of the `Search` variant in `LedgerCommands` and command routing.
- `src/commands/mod.rs`: Export the new module.
- `tests/ledger_search.rs`: Integration tests for the new search functionality.

## 3. Requirements

### 3.1 Data Access (`src/ledger/db.rs`)
Implement the `search_ledger` method:
```rust
pub fn search_ledger(
    &self,
    query: &str,
    category: Option<&str>,
    days: Option<u64>,
    breaking_only: bool,
) -> Result<Vec<LedgerEntry>, LedgerError>
```
- Query must join `ledger_entries` (alias `l`) with `ledger_fts` (alias `f`).
- `WHERE f.ledger_fts MATCH ?1`
- If `category` is provided: `AND l.category = ?2`
- If `days` is provided: `AND l.committed_at >= datetime('now', '-? days')`
- If `breaking_only` is true: `AND l.is_breaking = 1`
- Rank by FTS5 match score and deterministic fallback: `ORDER BY f.rank, l.committed_at DESC`
- Ensure the query handles FTS special characters gracefully (e.g., catching SQLite syntax errors and returning a clear user-facing `LedgerError`, or escaping double quotes to prevent arbitrary FTS syntax injection).

### 3.2 Command Logic (`src/commands/ledger_search.rs`)
- Define `execute_ledger_search(query: String, category: Option<String>, days: Option<u64>, breaking_only: bool) -> miette::Result<()>`.
- Fetch connection from `StorageManager`.
- Initialize `LedgerDb` and call `search_ledger`.
- Present the results in a clean list or table format. Include:
  - `committed_at`
  - `category`
  - `change_type`
  - `entity`
  - `summary`
  - `is_breaking` metadata.
- Provide a clear message if no results are found (e.g., "No ledger entries found matching the query.").

### 3.3 CLI Definitions (`src/cli.rs`)
Extend `LedgerCommands` with the `Search` variant:
```rust
    /// Search the ledger using full-text search
    Search {
        /// The search query
        query: String,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Only search entries from the last N days
        #[arg(long)]
        days: Option<u64>,
        /// Only show breaking changes
        #[arg(long)]
        breaking_only: bool,
    },
```

Add routing logic in the `run()` function:
```rust
LedgerCommands::Search { query, category, days, breaking_only } => {
    crate::commands::ledger_search::execute_ledger_search(query, category, days, breaking_only)
}
```

## 4. Testing
- `tests/ledger_search.rs`:
  - Create transactions, commit them.
  - Test FTS search round trip (insert -> search -> verify results).
  - Test category filter.
  - Test date filter (`days`).
  - Test `breaking_only` filter.
  - Test FTS result ranking order (ensure deterministic sorting by `rank, committed_at DESC`).

# Track L4-1: Transaction-Linked ADR Generation

## Objective
Implement the `ledger adr` command to export architectural decisions and breaking changes as MADR-format markdown files, as defined in Phase L4 of `docs/Ledger-Incorp-plan.md`.

## Deliverables
- `src/commands/ledger_adr.rs`: Implementation of the `ledger adr` command.
- `src/ledger/adr.rs`: Core logic for formatting MADR templates and generating filenames.
- `src/ledger/db.rs` & `src/ledger/transaction.rs`: Helpers to fetch entries where `entry_type = ARCHITECTURE` or `is_breaking = 1`.
- CLI routing updates in `src/cli.rs` and `src/commands/mod.rs`.
- Integration tests in `tests/ledger_adr.rs`.

## Architecture & Design Details

### 1. Data Retrieval
We need a targeted query to fetch ADR-eligible entries. 
In `src/ledger/db.rs`, add a new function `get_adr_entries(days: Option<u64>) -> Result<Vec<LedgerEntry>, LedgerError>`.
The SQL query should roughly be:
```sql
SELECT id, tx_id, category, entry_type, entity, entity_normalized, change_type, summary, reason, is_breaking, committed_at, verification_status, verification_basis, outcome_notes 
FROM ledger_entries 
WHERE (entry_type = 'ARCHITECTURE' OR is_breaking = 1)
```
If `days` is Some(N), append `AND committed_at >= datetime('now', '-N days')`.
Results must be sorted by `id` ASC or `committed_at` ASC.

Add a corresponding `get_adr_entries` wrapper to `TransactionManager` in `src/ledger/transaction.rs`.

### 2. MADR Template & Formatting
Create `src/ledger/adr.rs` to handle the generation of MADR content and filenames.
Template format based on `docs/Ledger-Incorp-plan.md`:
```markdown
# {id}. {summary}

- **Status**: {change_type}
- **Category**: {category}
- **Breaking**: {is_breaking}

## Context
{reason}

## Decision
{summary}

## Consequences
{If is_breaking == true, append consequence notes or a standard breaking warning, else "None."}
```
*Note*: `is_breaking` will be displayed as `Yes` or `No` (or `true`/`false`).
Filename logic: `{id}-{kebab-case-summary}.md`. We need a small utility function to strip non-alphanumeric characters, convert spaces to hyphens, and lowercase the summary.

### 3. Command Implementation
`src/commands/ledger_adr.rs` will house `execute_ledger_adr(output_dir: String, days: Option<u64>)`.
Workflow:
1. Initialize `StorageManager` to get a connection.
2. Instantiate `TransactionManager`.
3. Fetch eligible entries: `manager.get_adr_entries(days)?`.
4. If no entries found, log a message and return successfully.
5. `std::fs::create_dir_all(&output_dir)?` to ensure the directory exists.
6. Iterate over entries, generate MADR content and filename, and write files to the target directory.
7. Print a summary (e.g., "Exported N ADRs to docs/adr").

### 4. CLI Routing
Update `LedgerCommands` in `src/cli.rs` with the `Adr` variant:
```rust
    /// Export architectural decision records (MADR format)
    Adr {
        /// Directory to output ADRs
        #[arg(long, default_value = "docs/adr")]
        output_dir: String,
        /// Limit to entries from the last N days
        #[arg(long)]
        days: Option<u64>,
    },
```
Update the `match command` block in `src/cli.rs` to call `crate::commands::ledger_adr::execute_ledger_adr(output_dir, days)`.

### 5. Testing
Create `tests/ledger_adr.rs`:
- Seed a ledger database with mixed entry types (Implementation, Architecture) and `is_breaking` flags.
- Execute `execute_ledger_adr` pointing to a temporary directory.
- Verify that only Architecture and Breaking entries are exported.
- Verify file naming format and template contents.
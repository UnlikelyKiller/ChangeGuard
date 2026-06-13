# Technical Specification: Restore Ledger Note

## Context
The `ledger note` command was recently removed or deprecated but is required to satisfy agent workflows documented in `SKILL.md`. It must be restored with a clean UX, allowing either a positional note or a `--message` flag.

## Interface Contract

### CLI Arguments (`src/cli/args.rs`)
Extend `LedgerCommands` with a `Note` variant:
```rust
    /// Add a lightweight note/lesson to a transaction for an entity
    Note {
        /// Entity path
        entity: String,
        /// The note content
        #[arg(required_unless_present = "message")]
        note: Option<String>,
        /// The note content (takes precedence over positional note)
        #[arg(short, long)]
        message: Option<String>,
    },
```
*Note: Any previous "deprecated" help text must be omitted.*

### Execution Logic (`src/commands/ledger/lifecycle.rs`)
Implement `pub fn execute_ledger_note(entity: &str, note: Option<String>, message: Option<String>) -> Result<()>`:
1. Determine the final message text. If `message` is `Some`, use it. Otherwise, use `note`. If both are `None`, return a `miette!` error (though `clap` should prevent this, double-checking inside the function is defensive).
2. Call `TransactionManager::atomic_change` with:
   - `TransactionRequest { category: Category::Chore, entity: entity.to_string(), ..Default::default() }`
   - `CommitRequest { change_type: ChangeType::Modify, summary: final_message, reason: "Lightweight note".to_string(), ..Default::default() }`
   - `force: false`
3. Print a success message (e.g., `println!("{}", "Note recorded.".green().bold());`).

### Dispatch Logic (`src/cli/dispatch.rs`)
Update `dispatch_ledger` to match `LedgerCommands::Note` and route to `crate::commands::ledger::execute_ledger_note`. Ensure `src/commands/ledger.rs` (or equivalent module export) re-exports `execute_ledger_note`.

## Verification Strategy
- **Unit Tests**: Ensure `tests/integration/ledger_cli_parsing.rs` verifies the mutually inclusive argument structure (`note` OR `--message`). Test that `--message` takes precedence if both are passed.
- **Cargo Checks**: `cargo clippy` and `cargo test` pass.

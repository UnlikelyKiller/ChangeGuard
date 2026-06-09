# Specification: Track L3-R - Enforcement Remediation

## Objective
Address the findings from the Codex review for Track L3-1. This includes ensuring correct JSON deserialization of minimal payloads, CLI alignment with specifications, proper database constraints, and adequate validation of registered rules.

## Deliverables

### 1. JSON Deserialization (`src/ledger/enforcement.rs`)
Add `#[serde(default)]` and default-provider functions to enforcement structs so that users can register minimal JSON payloads as documented.
- `TechStackRule`: Default `rules` (`vec![]`), `locked` (`false`), `status` (`"ACTIVE"`), `entity_type` (`"FILE"`), and `registered_at` (can default to an empty string and be populated during registration, or a default date string).
- `CommitValidator`: Default `timeout_ms` (`30000`), `enabled` (`true`), `validation_level` (`ValidationLevel::Error`).
- `WatcherPattern`: Default `source` (`"CONFIG"`).
Add the necessary provider functions (e.g., `fn default_status() -> String`, `fn default_timeout() -> i32`).

### 2. CLI Alignment (`src/cli.rs`)
- **Ledger Register:** Change `rule_type` and `payload` in `LedgerCommands::Register` from positional arguments to flags (`--rule-type`, `--payload`). Use `crate::ledger::enforcement::RuleType` with clap's `value_enum` for the `rule_type` argument.
- **Ledger Stack:** Add an optional `--category` flag to `LedgerCommands::Stack` to filter the output.
- Update the router in `run()` to match the modified command signatures.

### 3. Registration Validation (`src/commands/ledger_register.rs`)
- Update `execute_ledger_register` to accept `rule_type: RuleType` instead of `String`.
- Add validation to reject malformed payloads before they reach the database:
  - `TechStackRule`: Reject empty `category` and `name`.
  - `CommitValidator`: Reject empty `category`, `name`, `executable`, and `timeout_ms <= 0`.
  - `CategoryStackMapping`: Reject empty `ledger_category` and `stack_category`.
  - `WatcherPattern`: Reject empty `glob` and `category`.
- Dynamically populate `registered_at` for `TechStackRule` if it's empty (e.g., using `chrono::Utc::now().to_rfc3339()`) so minimal payloads persist successfully.

### 4. Foreign Key Enforcement (`src/state/storage.rs`)
- Update `StorageManager::init` to include `PRAGMA foreign_keys = ON;` in the initial `execute_batch` call. This will properly enforce the references for `category_stack_mappings` and other tables.

### 5. Data Filtering (`src/ledger/db.rs` & `src/commands/ledger_stack.rs`)
- Update `get_tech_stack_rules` to take an optional filter: `pub fn get_tech_stack_rules(&self, category: Option<&str>) -> Result<Vec<TechStackRule>, LedgerError>`.
- Update `get_commit_validators` to take an optional filter: `pub fn get_commit_validators(&self, category: Option<&str>) -> Result<Vec<CommitValidator>, LedgerError>`.
- Modify the SQL queries within these methods to include a `WHERE category = ?1` clause when the filter is `Some`.
- Update `execute_ledger_stack` to accept `category: Option<String>`, passing it down to the DB methods, ensuring the CLI can show scoped views of the stack.

### 6. Tests (`tests/ledger_enforcement.rs`)
- Add tests to ensure JSON minimal payloads can be deserialized properly (exercising serde defaults).
- Add tests to ensure invalid payloads (e.g., empty fields, zero timeout) are rejected by the registration validation.
- Add tests for category filtering via the stack command logic.
- Add tests to verify `--force` behavior on locked rules during registration.
- Add tests to verify SQLite foreign key rejection (e.g., mapping to a nonexistent stack category).
- Ensure no production `unwrap()` remains in the updated files.

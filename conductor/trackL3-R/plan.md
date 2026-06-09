# Plan: Enforcement Remediation
### Phase 1: Storage and Schema Alignment
- [ ] Task 1.1: Update `StorageManager::init` in `src/state/storage.rs` to include `PRAGMA foreign_keys = ON;` in the batch execution.
- [ ] Task 1.2: Add `#[serde(default)]` and related default provider functions to `TechStackRule`, `CommitValidator`, `CategoryStackMapping`, and `WatcherPattern` in `src/ledger/enforcement.rs`.
- [ ] Task 1.3: Review `src/ledger/enforcement.rs` to ensure structs derive or provide appropriate default implementations for optional fields.

### Phase 2: Database Layer Updates
- [ ] Task 2.1: Update `get_tech_stack_rules` in `src/ledger/db.rs` to take `category: Option<&str>` and append a `WHERE category = ?1` clause if specified.
- [ ] Task 2.2: Update `get_commit_validators` in `src/ledger/db.rs` to take `category: Option<&str>` and append a `WHERE category = ?1` clause if specified.

### Phase 3: CLI Definition and Commands
- [ ] Task 3.1: Modify `LedgerCommands::Register` in `src/cli.rs` to use `--rule-type` (enum `RuleType`) and `--payload` flags instead of positional strings.
- [ ] Task 3.2: Modify `LedgerCommands::Stack` in `src/cli.rs` to accept an optional `--category` flag.
- [ ] Task 3.3: Update `run()` in `src/cli.rs` to pass the new parameters to the respective command execution handlers.

### Phase 4: Command Handlers and Validation
- [ ] Task 4.1: Update `execute_ledger_register` in `src/commands/ledger_register.rs` to pattern match on the `RuleType` enum directly.
- [ ] Task 4.2: Implement basic field validation in `execute_ledger_register` (reject empty strings, negative timeouts) before database insertion.
- [ ] Task 4.3: Ensure `registered_at` in `TechStackRule` is populated with the current UTC timestamp if it was left empty by deserialization.
- [ ] Task 4.4: Update `execute_ledger_stack` in `src/commands/ledger_stack.rs` to accept `category: Option<String>` and pass it to the updated DB methods.
- [ ] Task 4.5: Ensure no production `unwrap()` calls exist in `ledger_register.rs` and `ledger_stack.rs`.

### Phase 5: Testing
- [ ] Task 5.1: Enhance `tests/ledger_enforcement.rs` with tests verifying `TechStackRule` and `CommitValidator` default values when parsing minimal JSON payloads.
- [ ] Task 5.2: Add tests in `tests/ledger_enforcement.rs` to verify that invalid payloads are properly rejected by the handler's validation logic.
- [ ] Task 5.3: Add tests to verify foreign key constraint errors when inserting a category mapping for an invalid stack category.
- [ ] Task 5.4: Add tests to ensure category filtering works correctly when fetching rules and validators from the DB.
- [ ] Task 5.5: Add tests to verify `--force` correctly overrides locked tech stack rules.

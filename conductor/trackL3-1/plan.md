## Plan: Track L3-1 - Enforcement Data Model & Registration

### Phase 1: Data Model & Database Migrations
- [ ] Task 1.1: Create `src/ledger/enforcement.rs` and implement the types `RuleType` (enum: `TechStack`, `Validator`, `Mapping`, `Watcher`), `TechStackRule`, `ValidationLevel`, `CommitValidator`, and payload structs for mappings and watchers.
- [ ] Task 1.2: Update `src/state/migrations.rs` to add Migration `M13`. This must create the `tech_stack`, `commit_validators`, `category_stack_mappings`, and `watcher_patterns` tables.
- [ ] Task 1.3: Add a unit test in `src/state/migrations.rs` to verify that `M13` creates the tables correctly and allows inserting JSON lists (e.g., `rules`, `args`) properly.
- [ ] Task 1.4: Update `src/ledger/db.rs` with data access methods (`insert_tech_stack_rule`, `get_tech_stack_rules`, `insert_commit_validator`, `get_commit_validators`, `insert_category_mapping`, `insert_watcher_pattern`). Make sure arrays (`rules`, `args`) are correctly serialized/deserialized to JSON strings during SQLite insertions.

### Phase 2: Registration Commands & Routing
- [ ] Task 2.1: Update `src/cli.rs` and `LedgerCommands` to add the `Register` subcommand (taking `--rule-type` and `--payload`) and the `Stack` subcommand (taking optional `--category`).
- [ ] Task 2.2: Create `src/commands/ledger_register.rs`. Implement the `execute_ledger_register` function, which handles parsing the `--payload` JSON based on the `--rule-type` and inserting it via the `ledger::db` module. Include validation to prevent malformed payloads.
- [ ] Task 2.3: Create `src/commands/ledger_stack.rs`. Implement `execute_ledger_stack` to retrieve and format registered rules (`tech_stack` rules, and optionally `commit_validators`).
- [ ] Task 2.4: Update the `run()` match statement in `src/cli.rs` (or dispatch logic in `src/commands/ledger.rs`) to dispatch `LedgerCommands::Register` and `LedgerCommands::Stack` to their respective execution handlers.

### Phase 3: Testing & Verification
- [ ] Task 3.1: Create `tests/ledger_enforcement.rs` to establish a TDD verification loop.
- [ ] Task 3.2: Write tests for `ledger register` with valid JSON payloads for each `RuleType`.
- [ ] Task 3.3: Write tests for `ledger register` to ensure it rejects invalid JSON payloads or payloads that don't match the schema for the requested `RuleType`.
- [ ] Task 3.4: Verify `ledger stack` output includes recently registered tech stack items.

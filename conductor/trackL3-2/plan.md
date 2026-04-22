# Plan: Track L3-2 Enforcement & Validation Logic

### Phase 1: DB API & CLI Refinements
- [ ] Task 1.1: Update `src/ledger/db.rs`'s `get_category_mappings` signature to accept `category: Option<&str>`.
- [ ] Task 1.2: Update the SQL query in `get_category_mappings` to filter by `ledger_category` when `category` is provided.
- [ ] Task 1.3: Update `src/commands/ledger_stack.rs` to pass `category.as_deref()` to `db.get_category_mappings()`.
- [ ] Task 1.4: Update `src/commands/ledger_register.rs` to validate `validator.category` and `pattern.category` against empty trims.
- [ ] Task 1.5: Add `RuleViolation(String)` and `ValidatorFailed(String)` variants to `LedgerError` in `src/ledger/error.rs`.

### Phase 2: Validator Module implementation
- [ ] Task 2.1: Create `src/ledger/validators.rs` and define the `run_commit_validators` function.
- [ ] Task 2.2: Add logic to iterate over active `validators`, performing `{entity}` token substitution in `args` arrays.
- [ ] Task 2.3: Integrate `ExecutionBoundary::execute` to run the underlying command with the configured `timeout_ms`.
- [ ] Task 2.4: Interpret outcomes mapped against `ValidationLevel` (Error vs Warning), producing terminal output or rejecting with `LedgerError::ValidatorFailed`.
- [ ] Task 2.5: Register the `validators` module in `src/ledger/mod.rs`.

### Phase 3: Transaction Lifecycle Hooks
- [ ] Task 3.1: In `src/ledger/transaction.rs`, inside `start_change()`, look up mappings via `get_category_mappings(Some(&req.category))`.
- [ ] Task 3.2: Iterate over mapped stack categories to gather active `TechStackRule`s.
- [ ] Task 3.3: Implement the heuristic case-insensitive `NO <term>` check against `req.planned_action`, returning `LedgerError::RuleViolation` if caught.
- [ ] Task 3.4: Inside `commit_change()`, after verifying the transaction exists and before marking `COMMITTED`, look up relevant `CommitValidator`s.
- [ ] Task 3.5: Pass the extracted validators and absolute entity path to `run_commit_validators()`. If it fails, bubble the error to prevent the SQLite transaction from finalizing.

### Phase 4: Integration Tests & TDD
- [ ] Task 4.1: Write tests in `tests/ledger_enforcement.rs` verifying `NO <term>` functionality blocks `start_change`.
- [ ] Task 4.2: Write tests validating `ERROR`-level validators properly cancel `commit_change` when they trigger an exit code > 0.
- [ ] Task 4.3: Write tests confirming `WARNING`-level validators exit > 0 but do NOT halt `commit_change`.
- [ ] Task 4.4: Clean up code warnings and format `cargo fmt`. Validate pipeline readiness via `cargo test`.
# Track L3-2: Enforcement & Validation Logic

## Objective
Implement Phase L3 tech stack enforcement at transaction start and commit validators at transaction commit, integrating these gates directly into the transaction lifecycle. Complete the logic described in `docs/Ledger-Incorp-plan.md`.

## Deliverables

### 1. Data Model & Error Updates
- **`src/ledger/error.rs`**: Add new `LedgerError` variants to represent enforcement failures cleanly via `miette`:
  - `RuleViolation(String)`: For tech stack rule violations.
  - `ValidatorFailed(String)`: For commit validators that fail at the `ERROR` level.

### 2. Transaction Lifecycle Integration (`src/ledger/transaction.rs`)
- **`start_change` Enforcement**:
  - Retrieve the tech stack mappings for the transaction's category (`req.category`) via `db.get_category_mappings(Some(&req.category))`.
  - Fetch `TechStackRule`s for each mapped stack category.
  - If the transaction request provides a `planned_action`, parse the rules for `NO <term>` patterns.
  - Perform a case-insensitive check: if `planned_action.to_lowercase()` contains `term.to_lowercase()`, reject the transaction and return `LedgerError::RuleViolation(...)`.
- **`commit_change` Validation**:
  - Retrieve `CommitValidator`s applicable to `tx.category` (using `db.get_commit_validators(Some(&tx.category))`).
  - Pass the retrieved validators and the transaction's absolute entity path to the new validator runner module (`src/ledger/validators.rs`).
  - If a validator fails with an `ERROR` level, bubble up the `LedgerError::ValidatorFailed(...)` error, effectively blocking the commit and rolling back the DB transaction.

### 3. Validator Runner (`src/ledger/validators.rs`)
- Create a new module responsible for executing shell-based validators.
- Expose `pub fn run_commit_validators(validators: &[CommitValidator], entity_path: &str) -> Result<(), LedgerError>`.
- For each active validator:
  - Substitute the literal `{entity}` placeholder in `validator.args` with the given `entity_path`.
  - Use `crate::exec::ExecutionBoundary::execute` to run the `Command` with a bounded timeout (`Duration::from_millis(validator.timeout_ms as u64)`).
  - Interpret the `ExecutionResult`:
    - On non-zero exit or timeout:
      - If `validator.validation_level == ValidationLevel::Error`: Return a hard error (`LedgerError::ValidatorFailed`).
      - If `validator.validation_level == ValidationLevel::Warning`: Print a structured, dimmed/yellow warning message using `owo_colors` and continue executing the rest.

### 4. CLI & DB Refinements
- **`src/ledger/db.rs`**:
  - Modify `get_category_mappings(&self)` to `get_category_mappings(&self, category: Option<&str>)` to support targeted lookup.
  - When `category` is `Some(c)`, filter the SQL query via `WHERE ledger_category = ?1` or `WHERE ledger_category = ?1 OR ledger_category = 'ALL'`.
- **`src/commands/ledger_stack.rs`**:
  - Pass `category.as_deref()` to the updated `get_category_mappings(...)` function, fixing the missing filter capability.
- **`src/commands/ledger_register.rs`**:
  - Refine input validation in `RuleType::Validator`: check `validator.category.trim().is_empty()`.
  - Refine input validation in `RuleType::Watcher`: check `pattern.category.trim().is_empty()`.

### 5. Testing & TDD (`tests/ledger_enforcement.rs`)
- Follow TDD by writing integration tests covering:
  - **Start Rejection**: A registered `NO <term>` rule correctly blocks a violating `start_change`.
  - **Commit Error Validator**: An `ERROR`-level validator that exits non-zero (e.g., `executable: "false"`) blocks `commit_change`.
  - **Commit Warning Validator**: A `WARNING`-level validator that exits non-zero produces a warning but permits the commit to succeed.
  - **Timeout Handling**: A validator configured to exceed its timeout is aborted and correctly triggers its validation level policy.
- Ensure deterministic testing via stable environment variables and predictable validator commands.

## Architecture & Boundary Notes
- Validators strictly execute as direct process invocations (`Command::new().args()`). They do not utilize shell evaluation unless explicitly requested (e.g., `executable: "sh", args: ["-c", ...]`). This prevents command injection vulnerabilities from crafted entity paths.
- The `planned_action` regex matching must be heuristic but reliable: `NO <term>` should be stripped of `"NO "` and matched strictly on the remaining `<term>` in lowercase.
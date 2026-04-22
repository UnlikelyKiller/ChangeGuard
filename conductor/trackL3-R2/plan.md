## Plan: Enforcement Logic Remediation (Track L3-R2)
### Phase 1: Error Variants and Database Queries
- [ ] Task 1.1: Update `src/ledger/error.rs` to add `RuleViolation(String)` and `ValidatorFailed(String, String)` variants to `LedgerError`, properly annotating them with `miette` and `thiserror` macros.
- [ ] Task 1.2: Update `src/ledger/db.rs` in the `get_commit_validators` method. Change the SQL query to include `category = 'ALL'` when a specific category is provided (e.g., `WHERE category = ?1 OR category = 'ALL'`).

### Phase 2: Transaction Lifecycle Integration
- [ ] Task 2.1: Update `TransactionManager::start_change` in `src/ledger/transaction.rs` to return `LedgerError::RuleViolation` instead of `LedgerError::Validation` when a tech stack rule (`NO <term>`) is violated.
- [ ] Task 2.2: Update `TransactionManager::commit_change` in `src/ledger/transaction.rs` to resolve the transaction's `entity` path against `repo_root` to produce an absolute path.
- [ ] Task 2.3: Pass the resolved absolute path to `run_commit_validators` in `commit_change`.
- [ ] Task 2.4: Update error handling in `commit_change` to return `LedgerError::ValidatorFailed` instead of `LedgerError::Validation` when an `ERROR` level validator fails. Ensure the new error takes the necessary parameters (e.g., validator name and error message).

### Phase 3: Testing
- [ ] Task 3.1: Add a test case in `tests/ledger_enforcement.rs` verifying that an `ERROR`-level validator which exceeds its timeout causes a validation failure and prevents commit.
- [ ] Task 3.2: Add a test case in `tests/ledger_enforcement.rs` verifying that `{entity}` substitution within validator arguments uses the correct absolute path of the entity, not just the relative path.

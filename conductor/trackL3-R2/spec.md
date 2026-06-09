# Track L3-R2: Enforcement Logic Remediation

## Objective
Remediate the findings from the Codex review for Track L3-2. Specifically, address issues related to path substitution for validators, global validator filtering, error handling granularity, and missing test coverage for validator timeouts and `{entity}` path substitution.

## Findings to Address
1. **Path Substitution (`src/ledger/transaction.rs`)**: Update `commit_change` to pass the absolute path of the entity (resolved against `repo_root`) to the `ValidatorRunner`, ensuring `{entity}` substitution provides a valid path for external tools.
2. **Global Validators (`src/ledger/db.rs`)**: Update `get_commit_validators` in `LedgerDb` to include validators where `category = 'ALL'` when a specific category filter is provided.
3. **Error Variants (`src/ledger/error.rs`, `src/ledger/transaction.rs`)**: Add `RuleViolation(String)` and `ValidatorFailed(String, String)` variants to `LedgerError` in `src/ledger/error.rs` and use them in `TransactionManager` to replace generic `LedgerError::Validation`.
4. **Testing - Timeout (`tests/ledger_enforcement.rs`)**: Add a test case to verify that validator timeouts correctly fail the validation or emit a warning based on their validation level.
5. **Testing - Entity Substitution (`tests/ledger_enforcement.rs`)**: Add a test case verifying that `{entity}` substitution uses the absolute path of the entity being committed.

## Deliverables
- Updates to `src/ledger/error.rs` to include the specific `RuleViolation` and `ValidatorFailed` variants.
- Updates to `src/ledger/db.rs` to fix category filtering for global validators.
- Updates to `src/ledger/transaction.rs` to resolve the absolute entity path and use the new error variants.
- Updates to `tests/ledger_enforcement.rs` to cover validator timeouts and path substitution logic.

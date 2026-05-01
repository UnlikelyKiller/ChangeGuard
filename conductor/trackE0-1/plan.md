## Plan: Track E0-1 Ledger Verification Gate Enforcement

### Phase 1: Define Enforcement Categories and Extend CommitRequest
- [ ] Task 1.1: Create a constant `VERIFICATION_REQUIRED_CATEGORIES` array in `src/ledger/transaction.rs` containing `Category::Architecture`, `Category::Feature`, `Category::Bugfix`, and `Category::Infra`.
- [ ] Task 1.2: Add `pub force: bool` field with `#[serde(default)]` defaulting to `false` to `CommitRequest` in `src/ledger/types.rs`.
- [ ] Task 1.3: Write failing test `test_verification_gate_blocks_high_risk_without_status` that starts a FEATURE transaction and attempts to commit without `verification_status`, expecting `VerificationRequired`.
- [ ] Task 1.4: Write failing test `test_verification_gate_rejects_status_without_basis` that provides `verification_status` but no `verification_basis`, expecting `VerificationRequired`.

### Phase 2: Implement Verification Gate in commit_change
- [ ] Task 2.1: Add `force: bool` parameter to `TransactionManager::commit_change` signature.
- [ ] Task 2.2: Insert the verification gate logic after the PENDING-status check and before commit validators. When `config.ledger.verify_to_commit` is true, the category is in `VERIFICATION_REQUIRED_CATEGORIES`, and `force` is false: check that `verification_status` is `Some` and `verification_basis` is `Some`. If `verification_status` is `None` OR `verification_status` is `Some` but `verification_basis` is `None`, return `LedgerError::VerificationRequired(category_str)`.
- [ ] Task 2.3: When `force` is true and the gate would have blocked, emit `tracing::warn!("Verification gate bypassed with --force for transaction {} (category: {:?})", tx_id, tx.category)`.
- [ ] Task 2.4: Run the failing tests from Phase 1. Confirm they now pass.

### Phase 3: Extend Coverage and Edge Cases
- [ ] Task 3.1: Write and pass test `test_verification_gate_allows_with_status_and_basis`: FEATURE commit with both fields set succeeds.
- [ ] Task 3.2: Write and pass test `test_verification_gate_skipped_when_config_disabled`: FEATURE commit without verification fields succeeds when `verify_to_commit = false`.
- [ ] Task 3.3: Write and pass test `test_verification_gate_skipped_for_low_risk_category`: DOCS commit without verification fields succeeds even when `verify_to_commit = true`.
- [ ] Task 3.4: Write and pass test `test_verification_gate_force_override`: FEATURE commit without verification fields but with `force = true` succeeds and emits a warning log.

### Phase 4: CLI Integration
- [ ] Task 4.1: Add `--force` flag to the `ledger commit` clap subcommand in the CLI argument definitions.
- [ ] Task 4.2: Update `execute_ledger_commit` in `src/commands/ledger.rs` to accept and forward the `force` flag to `tx_mgr.commit_change`.
- [ ] Task 4.3: Add `--force` flag to the `ledger atomic` clap subcommand in `src/commands/ledger.rs`.
- [ ] Task 4.4: Update `execute_ledger_atomic` to accept and forward `force` through `CommitRequest::force` to `commit_change`.
- [ ] Task 4.5: Update `atomic_change` in `src/ledger/transaction.rs` to accept and forward the `force` parameter to `commit_change`.

### Phase 5: Regression and Integration Verification
- [ ] Task 5.1: Run `cargo test` and confirm all existing tests pass.
- [ ] Task 5.2: Run `cargo clippy` and resolve any new warnings.
- [ ] Task 5.3: Manual smoke test: `changeguard ledger start src/main.rs --category FEATURE`, then `changeguard ledger commit <tx_id> --summary "test" --reason "test" --change-type MODIFY`. Verify it fails with a verification required error.
- [ ] Task 5.4: Manual smoke test: same as above with `--force`. Verify it succeeds and logs a warning.
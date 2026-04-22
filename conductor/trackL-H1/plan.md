## Plan: Ledger Production Hardening (Track L-H1)

### Phase 1: Ledger Lifecycle Invariants
- [ ] Task 1.1: Update `src/state/migrations.rs` (Migration M11) to add a `UNIQUE INDEX` on `(entity_normalized, status) WHERE status = 'PENDING'`.
- [ ] Task 1.2: Update `src/ledger/db.rs` function `update_transaction_status` to append `AND status = 'PENDING'` to the `WHERE` clause.
- [ ] Task 1.3: Ensure `update_transaction_status` returns the affected row count and update callers to handle 0-row updates as concurrency failures.

### Phase 2: Durable State Protection
- [ ] Task 2.1: Modify `src/cli.rs` to add an `--include-ledger` flag to the `Reset` subcommand.
- [ ] Task 2.2: Update `src/commands/reset.rs` to exclude `ledger.db` from the default deletion behavior.
- [ ] Task 2.3: Wire the `--include-ledger` flag in `src/commands/reset.rs` to allow explicit deletion of `ledger.db` when requested.

### Phase 3: Secure Path Normalization
- [ ] Task 3.1: Create `src/util/path.rs` with the `normalize_relative_path` utility function (implementing lexical normalization and repo-root confinement without `canonicalize`).
- [ ] Task 3.2: Expose the `path` module in `src/util/mod.rs` (or create it if needed).
- [ ] Task 3.3: Refactor `TransactionManager` in `src/ledger/transaction.rs` to use `normalize_relative_path`.
- [ ] Task 3.4: Refactor `DriftManager` in `src/ledger/drift.rs` to use `normalize_relative_path` instead of ad-hoc fallback logic.
- [ ] Task 3.5: Refactor federation import logic in `src/ledger/federation.rs` to use `normalize_relative_path` instead of substring checks.

### Phase 4: Security, Policy, & Discovery
- [ ] Task 4.1: Update `ValidatorRunner` in `src/ledger/validators.rs` to use the shared `ProcessPolicy` for executing commands, replacing direct `Command::new` usage.
- [ ] Task 4.2: Update federate commands (e.g., in `src/commands/federate.rs` or relevant entry points) to discover the git repo root before constructing `Layout` and `FederatedScanner`.
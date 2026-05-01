# Specification: Track E0-1 Ledger Verification Gate Enforcement

## Overview

Categories like ARCHITECTURE, FEATURE, BUGFIX, and INFRA are intended to require
`verification_status` and `verification_basis` at commit time, but the
`commit_change` method in `src/ledger/transaction.rs` does not enforce this.
The `LedgerError::VerificationRequired` variant already exists in
`src/ledger/error.rs` but is never returned. This track closes the gap by
adding enforcement logic inside `commit_change`, gated behind the existing
`config.ledger.verify_to_commit` flag, with a `--force` override on the CLI.

## Components

### 1. Verification Gate Logic (`src/ledger/transaction.rs`)

Add a check inside `commit_change`, after the PENDING-status check and before
commit validators run. The gate must:

- Define which categories require verification. The enforcement set is:
  `Architecture`, `Feature`, `Bugfix`, `Infra`. Categories `Refactor`,
  `Tooling`, `Docs`, and `Chore` are exempt because they carry lower risk.
- When `config.ledger.verify_to_commit` is `true` and the transaction's
  category is in the enforcement set, reject the commit if
  `req.verification_status` is `None` by returning
  `LedgerError::VerificationRequired(category_str)`.
- When `config.ledger.verify_to_commit` is `true` and the category is in the
  enforcement set, also reject if `verification_status` is `Some` but
  `verification_basis` is `None`. A status without a basis is incomplete.
  The check must cover both: `verification_status.is_none()` **and**
  `verification_status.is_some() && verification_basis.is_none()`.
- When `config.ledger.verify_to_commit` is `false`, skip the gate entirely
  (current behavior). This ensures no breaking change to existing workflows
  that have not opted in.
- Accept a `force: bool` parameter. When `force` is `true`, bypass the
  verification gate regardless of config or category. Log a warning via
  `tracing::warn!` when the gate is bypassed with force, using the format:
  `tracing::warn!("Verification gate bypassed with --force for transaction {} (category: {:?})", tx_id, tx.category);`

### 2. CommitRequest Extension (`src/ledger/types.rs`)

Add a `force: bool` field to `CommitRequest` with `#[serde(default)]` defaulting
to `false`. This preserves backward compatibility for programmatic callers that
construct `CommitRequest` without the field.

### 3. TransactionManager Signature Change (`src/ledger/transaction.rs`)

Update `commit_change` to accept the `force` flag. Two approaches, pick one:

- **Option A (preferred)**: Add `force: bool` parameter to `commit_change`.
  Callers pass it explicitly.
- **Option B**: Read `force` from the `CommitRequest` struct added in component 2.

Option A is preferred because it keeps the method signature clear and avoids
mixing policy flags into the data transfer object.

### 4. CLI `--force` Flag (`src/commands/ledger.rs`)

Add a `--force` boolean flag to the `ledger commit` subcommand. When present,
pass `force: true` through to `TransactionManager::commit_change`. The flag must
also be accepted by `execute_ledger_commit` and forwarded.

### 5. CLI `--force` Flag for `ledger atomic` (`src/commands/ledger.rs`)

The `atomic_change` method also calls `commit_change`. Add `--force` to
`ledger atomic` as well, forwarding it through to the underlying commit.

### 6. Unit Tests (`tests/ledger_enforcement.rs`)

Add integration tests in `tests/ledger_enforcement.rs`:

- `test_verification_gate_blocks_high_risk_categories`: Start ARCHITECTURE,
  FEATURE, BUGFIX, and INFRA transactions without `verification_status`,
  attempt commit, expect `VerificationRequired` for each category. Roll back
  between iterations.
- `test_verification_gate_allows_with_status`: Start an ARCHITECTURE transaction,
  commit with `verification_status: Some(Verified)` and
  `verification_basis: Some(Tests)`, expect success.
- `test_verification_gate_force_override`: Start a FEATURE transaction without
  verification fields, commit with `force = true` and `verify_to_commit = true`,
  expect success.
- `test_verification_gate_disabled_by_default`: With default config
  (`verify_to_commit = false`), commit an ARCHITECTURE transaction without
  verification fields, expect success.
- `test_verification_gate_allows_low_risk_categories`: Commit DOCS, CHORE,
  REFACTOR, and TOOLING transactions without verification fields while
  `verify_to_commit = true`, expect success for all (exempt categories).

## Constraints & Guidelines

- **No breaking changes**: The gate is off by default (`verify_to_commit`
  defaults to `false`). Existing users who have not set `verify_to_commit = true`
  see zero behavior change.
- **Error variant reuse**: Use the existing `LedgerError::VerificationRequired`
  variant. Do not add a new error variant.
- **Config-gated**: The gate must respect `config.ledger.verify_to_commit`. Do
  not introduce a separate config key.
- **Force log**: Every forced bypass must emit a `tracing::warn!` with the
  transaction ID and category for auditability.
- **Atomic change**: The `atomic_change` method must also respect the force
  flag. Its internal `commit_change` call must forward `force`.
- **TDD**: Write tests first, confirm they fail, then implement the gate.

## Acceptance Criteria

1. When `verify_to_commit = true` in config, committing a transaction with
   category ARCHITECTURE, FEATURE, BUGFIX, or INFRA without
   `verification_status` returns `LedgerError::VerificationRequired`.
2. When `verify_to_commit = true`, providing `verification_status` without
   `verification_basis` also returns `VerificationRequired`.
3. When `verify_to_commit = false` (the default), commits succeed regardless
   of verification fields (no behavior change).
4. The `--force` CLI flag on both `ledger commit` and `ledger atomic` bypasses
   the verification gate and logs a `tracing::warn!` with transaction ID and
   category.
5. Categories Refactor, Tooling, Docs, and Chore are exempt from the gate even
   when `verify_to_commit = true`.
6. All five unit tests in `tests/ledger_enforcement.rs` pass.
7. Existing integration tests continue to pass (no regressions).

## Definition of Done

- All acceptance criteria pass
- All unit tests pass
- `cargo fmt --all -- --check` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
- `cargo test` passes with no regressions
- No deviations from this spec without documented justification
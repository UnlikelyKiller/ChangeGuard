# Specification: Track L-H1 - Ledger Production Hardening

## Overview
This track addresses the critical and high-severity findings from the final Codex review for Milestone L. The goal is to harden the Ledger functionality to ensure production readiness. This involves fixing concurrent transaction issues, protecting durable ledger state from accidental deletion, enforcing secure and unified path normalization, ensuring process policy adherence for validators, and correctly discovering the repository root for federation.

## Detailed Requirements

### 1. Lifecycle Invariants (High Severity)
**Problem:** Concurrent `ledger start` can create multiple PENDING transactions. Concurrent `ledger commit` can result in double commits.
**Solution:**
- **Database Schema (Migrations):** Update the schema definition (e.g., Migration M11) in `src/state/migrations.rs` to add a `UNIQUE` index for the combination of `(entity_normalized, status)` where `status = 'PENDING'`. This database-level constraint prevents concurrent double-starts for the same entity.
- **Atomic Commits:** Update `update_transaction_status` in `src/ledger/db.rs`. The `UPDATE` statement must include a `status = 'PENDING'` clause in its `WHERE` condition. The commit logic should ensure the update was successful (affected rows > 0) to ensure atomicity and avoid double-commits.

### 2. Durable State Protection (High Severity)
**Problem:** `changeguard reset` deletes the durable ledger state (`ledger.db`) by default, leading to potential data loss.
**Solution:**
- **Reset Command Updates:** Update `src/commands/reset.rs` to explicitly exclude `ledger.db` from the default reset operation. The default reset should only clear ephemeral state (e.g., caches, intermediate representations) but preserve the ledger database.
- **CLI Flag:** Introduce a new flag `--include-ledger` to the `Reset` command definition in `src/cli.rs`. Only when this flag is explicitly provided should `ledger.db` be deleted during a reset.

### 3. Secure Path Normalization (High Severity)
**Problem:** Path normalization is inconsistent, ad hoc, and bypasses repository confinement when canonicalization fails or when importing federated entries.
**Solution:**
- **Central Utility:** Create a new central utility `src/util/path.rs`.
- **Function Contract:** Implement a `normalize_relative_path(repo_root: &Path, input: &Path) -> Result<PathBuf>` function that:
  - Joins `input` with `repo_root`.
  - Performs lexical normalization (e.g., resolving `..` and `.`) strictly through logical path components, *without* calling `fs::canonicalize()`. This ensures paths can be normalized even if files are deleted or non-existent.
  - Validates that the resulting path is strictly confined within `repo_root`.
- **Refactoring:** Update `src/ledger/transaction.rs` (TransactionManager), `src/ledger/drift.rs` (DriftManager), and `src/ledger/federation.rs` (importing federated entries) to replace existing ad-hoc path parsing/validation with calls to `normalize_relative_path`.

### 4. Security & Policy (Medium Severity)
**Problem:** `ValidatorRunner` invokes commands directly, bypassing the guarded `ProcessPolicy`.
**Solution:**
- **Process Policy Integration:** Refactor `ValidatorRunner` in `src/ledger/validators.rs` to use ChangeGuard's central `ProcessPolicy`. This ensures that execution of validators blocks dangerous commands and enforces configured positive timeouts consistently with other parts of the system.

### 5. Root Discovery (Medium Severity)
**Problem:** `federate` commands use the current directory instead of the git repository root.
**Solution:**
- **Git Root Discovery:** Update the federation scanning logic to correctly discover the git repository root before building the `Layout` and `FederatedScanner`. This ensures that even when run from a subdirectory, the scanner processes the correct relative paths and accesses the correct `.changeguard/state/ledger.db`.

## FFI/API Contracts
- `src/util/path.rs::normalize_relative_path(repo_root: &Path, input: &Path) -> Result<PathBuf, Error>`
  - Must return an error if the normalized path escapes the `repo_root`.
- `src/ledger/db.rs::update_transaction_status(...) -> Result<usize, Error>`
  - Must return the number of rows updated or an error. If zero rows are updated when expecting one, it indicates a concurrent modification or invalid state transition.

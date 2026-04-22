# Track L1-R Specification: Ledger Phase 1 Remediation

## Overview
This specification addresses the High, Medium, and Low severity findings from the Codex review for Phase L1 (Ledger Incorporation) of ChangeGuard. It ensures the local-first change intelligence ledger is robust, transactionally safe, correctly normalizes paths, and strictly conforms to the planned schema and states.

## Findings Addressed & Technical Design

### 1. Lifecycle State Correction (High)
**Issue:** `commit_change` transitions transactions to `RESOLVED` instead of `COMMITTED`.
**Remediation:** 
- In `src/ledger/transaction.rs`, update `commit_change` to set status to `COMMITTED`.
- In `tests/ledger_lifecycle.rs`, update assertions to check for `COMMITTED` instead of `RESOLVED`.

### 2. Transactional Safety (High)
**Issue:** `commit_change` and `atomic_change` do not execute inside a database transaction, leaving the system vulnerable to partial updates (e.g., status updated but ledger entry insertion fails).
**Remediation:**
- Modify `StorageManager` to expose `&mut Connection` or a transaction wrapper, allowing `rusqlite::Transaction` to be used.
- Update `TransactionManager::commit_change` to wrap the `update_transaction_status` and `insert_ledger_entry` calls within a single `rusqlite` transaction. If insertion fails, the status update rolls back.
- Update `atomic_change` to wrap both `start_change` and `commit_change` within a single SQLite transaction, ensuring no orphaned `PENDING` states on commit failure.

### 3. Path Normalization & State Root Discovery (High)
**Issue:** Path normalization is naive, allows directory traversal, does not strip UNC paths, and doesn't handle Windows case-folding correctly. Additionally, ledger commands use `current_dir` as the state root instead of the git repository root.
**Remediation:**
- **State Root:** In `src/commands/ledger.rs`, replace `env::current_dir()` with logic that discovers the git repository root. Use this repo root to initialize `Layout`.
- **Path Normalization:**
  - In `src/ledger/transaction.rs` (or a dedicated path util module), update `entity_normalized`:
    - Make paths absolute based on `current_dir`, then resolve them relative to the discovered git repo root.
    - Reject any paths that resolve outside the repository boundary (preventing `../outside.rs` attacks).
    - Strip the Windows UNC prefix `\\?\`.
    - Lowercase Windows drive letters (e.g., `C:` -> `c:`).
    - Conditionally case-fold paths to lowercase if the underlying filesystem is case-insensitive (probe by checking file metadata with alternate cases, or use `cfg!(windows)`/`cfg!(target_os = "macos")` as a heuristic if probing fails).

### 4. Concurrency Protections (Medium)
**Issue:** SQLite is not configured for high concurrency.
**Remediation:**
- In `src/state/storage.rs`, within `StorageManager::init`, execute:
  ```sql
  PRAGMA journal_mode = WAL;
  PRAGMA busy_timeout = 5000;
  ```
  This must run right after opening the SQLite connection.

### 5. Verification Lifecycle Fields (Medium)
**Issue:** `verification_status`, `verification_basis`, and `outcome_notes` are defined in `CommitRequest` but ignored during commit.
**Remediation:**
- In `src/ledger/transaction.rs` and `src/ledger/db.rs`, ensure that when `commit_change` is called, the verification fields from `CommitRequest` are validated and persisted either in the `transactions` table update or the `ledger_entries` insert (depending on schema definition in `docs/Ledger-Incorp-plan.md`). Ensure missing verification for major categories (e.g., Feature, Bugfix) is rejected or properly defaulted.

### 6. CLI Gaps (Medium)
**Issue:** Incomplete CLI functionality for `rollback`, `status`, and `resume`.
**Remediation:**
- **Rollback:** Add `--reason` argument to `LedgerCommands::Rollback` in `src/cli.rs`. Pass this to `TransactionManager::rollback_change` to log why the rollback occurred.
- **Status:** 
  - Add `--compact` flag to `LedgerCommands::Status`.
  - In `src/commands/ledger.rs`, if `--entity` is omitted, query and display a global view of all `PENDING` transactions. Use `--compact` to format output densely.
- **Resume:** Make `tx_id` optional in `LedgerCommands::Resume`. If omitted, query the database for the most recently started `PENDING` transaction in the current repository and resume it.

### 7. Code Quality & Security (Low)
**Issue:** Loose wildcard matching in fuzzy resolver and unhandled `unwrap()` calls.
**Remediation:**
- In `src/ledger/db.rs`, `resolve_tx_id_fuzzy`, escape `%` and `_` characters in the user-provided prefix before appending the wildcard `%` for the `LIKE` clause.
- Remove all instances of `.unwrap()` in enum serialization in `src/ledger/db.rs` (e.g., `serde_json::to_string(&tx.category).unwrap()`). Use `.map_err(...)` to convert serialization errors into `LedgerError`.

### Testing Requirements
Enhance `tests/ledger_lifecycle.rs` to cover:
- Commit state asserting `COMMITTED`.
- Database transaction rollback on failed commit insert.
- Absolute path normalization, traversal rejection, and case-folding on Windows.
- Global status and compact output rendering.
- Resume by most recent context (no args).
- Verification fields persistence.

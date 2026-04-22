# Track L1-R Plan: Ledger Phase 1 Remediation

## Phase 1: Core State & Transaction Safety Fixes
- [x] Task 1.1: **Update Lifecycle State**
  - Change `RESOLVED` to `COMMITTED` in `TransactionManager::commit_change` (`src/ledger/transaction.rs`).
  - Update `assert_eq!(tx.status, "RESOLVED");` to check for `"COMMITTED"` in `tests/ledger_lifecycle.rs`.
- [x] Task 1.2: **Enable SQLite Transactional Safety**
  - Update `StorageManager` in `src/state/storage.rs` to expose `&mut Connection` or a wrapper that provides transaction access.
  - Refactor `TransactionManager` to take `&mut Connection` (or use internal mutability if necessary) so it can use `rusqlite::Transaction`.
  - Wrap `commit_change` database operations (update status + insert ledger entry) in a single SQLite transaction.
  - Wrap `atomic_change` operations (start + commit) in a single SQLite transaction.

## Phase 2: Path Normalization & State Root Discovery
- [x] Task 2.1: **State Root Discovery**
  - In `src/commands/ledger.rs`, modify `get_layout()` to discover the git repository root instead of using `env::current_dir()`.
  - Use the discovered repo root to instantiate the `Layout`.
- [x] Task 2.2: **Secure Path Normalization**
  - Rewrite `entity_normalized` in `src/ledger/transaction.rs` (or extract to a `path_utils` module).
  - Resolve the input path absolutely against `env::current_dir()`, then find its relative path from the git repo root.
  - Validate that the path does not escape the repository root.
  - Strip `\\?\` UNC prefixes on Windows.
  - Lowercase Windows drive letters.
  - Implement filesystem case-sensitivity probing (or platform-specific defaults) and case-fold the path if on a case-insensitive filesystem.

## Phase 3: Database Robustness & Code Quality
- [x] Task 3.1: **Enable Concurrency Protections**
  - Add `conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")?` to `StorageManager::init` in `src/state/storage.rs`.
- [x] Task 3.2: **Remove unwrap() and Secure Fuzzy Matching**
  - Replace `.unwrap()` calls in `src/ledger/db.rs` enum serialization with `.map_err()` returning a `LedgerError`.
  - In `resolve_tx_id_fuzzy` (`src/ledger/db.rs`), properly escape `_` and `%` characters in the user's input prefix before executing the `LIKE` query.

## Phase 4: Feature Completeness & CLI Gaps
- [x] Task 4.1: **Verification Fields Persistence**
  - Update `TransactionManager::commit_change` to validate and process `verification_status`, `verification_basis`, and `outcome_notes` from `CommitRequest`.
  - Persist these fields in the database as per the L1 plan schema.
- [x] Task 4.2: **CLI Fixes (Rollback, Resume, Status)**
  - Add `--reason` argument to `Rollback` in `src/cli.rs` and update `execute_ledger_rollback` to use it.
  - Make `tx_id` optional in `Resume` in `src/cli.rs`. Implement fallback to the most recent global `PENDING` transaction in `execute_ledger_resume`.
  - Add `--compact` flag to `Status` in `src/cli.rs`. Implement the global status view (all `PENDING` transactions) when no `--entity` is provided in `execute_ledger_status`.

## Phase 5: Verification & Testing
- [x] Task 5.1: **Enhance Test Coverage**
  - Add tests in `tests/ledger_lifecycle.rs` for:
    - Path traversal rejection.
    - Absolute path normalization (and Windows specific normalization if applicable).
    - Database transaction rollback on failed commit insert.
    - Global status and resume without arguments.
    - Verification fields persistence.
  - Run all tests to verify remediation completeness.

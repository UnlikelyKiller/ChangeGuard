# Track L1-2 Specification: Transaction Lifecycle Management

## 1. Objective
Implement the core transaction lifecycle (start, commit, rollback, atomic, status, note, resume) for the ChangeGuard Ledger, as defined in Phase L1 of `docs/Ledger-Incorp-plan.md`. This establishes the foundation for tracking architectural changes.

## 2. Scope & Deliverables

### 2.1 Storage & Data Model
- **Target File**: `src/state/migrations.rs`
- **Details**: Implement Migration 11 (`M11`) for `transactions` and `M12` for `ledger_entries` + `ledger_fts` virtual table with corresponding SQLite triggers. Ensure the connection specifies `PRAGMA journal_mode = WAL` and `PRAGMA busy_timeout = 5000` (which should already be handled in `StorageManager`).
- **Target File**: `src/ledger/db.rs`
- **Details**: SQLite operations for managing the transaction lifecycle and writing to ledger entries. Uses the existing `rusqlite::Connection` or shared `StorageManager`.

### 2.2 Core Logic
- **Target File**: `src/ledger/session.rs`
- **Details**: Generates and manages the `session_id` (process startup timestamp). Used to identify active transactions for the current CLI session vs stale/orphaned transactions.
- **Target File**: `src/ledger/transaction.rs`
- **Details**: The main API for lifecycle transitions (start, commit, rollback). Handles path normalization logic conditionally (case-folded on case-insensitive file systems like Windows NTFS/macOS APFS default, untouched on case-sensitive file systems like Linux ext4).

### 2.3 CLI Commands
New subcommands under `changeguard ledger`:
- **`src/commands/ledger.rs`**: `ledger start`
- **`src/commands/ledger_commit.rs`**: `ledger commit`
- **`src/commands/ledger_rollback.rs`**: `ledger rollback`
- **`src/commands/ledger_atomic.rs`**: `ledger atomic`
- **`src/commands/ledger_status.rs`**: `ledger status`
- **`src/commands/ledger_note.rs`**: `ledger note`
- **`src/commands/ledger_resume.rs`**: `ledger resume`
- **`src/cli.rs`**: Registration of the `ledger` command group and routing.

## 3. Technical Requirements

### 3.1 Path Normalization (`entity_normalized`)
Implement a deterministic path normalization strategy in `transaction.rs` (or a utility module):
1. Resolve relative to the workspace root.
2. Strip UNC prefix (`\\?\`) on Windows long paths.
3. Convert backslashes to forward slashes.
4. Lowercase drive letters on Windows.
5. Strip leading `./`.
6. Case-folding: Apply `.to_lowercase()` for `entity_normalized` ONLY on case-insensitive file systems (probe during `init` or heuristically). If case-sensitive, leave casing intact for `entity_normalized`.
7. Store the normalized path as `entity_normalized` for conflict detection, and original casing as `entity` for display.

### 3.2 Fuzzy UUID Matching
For `commit` and `rollback`, accept a truncated UUID (e.g., first 8 chars) if it uniquely matches exactly one `PENDING` transaction. If ambiguous, fail deterministically and print the candidate list.

### 3.3 Transaction Integrity
- **Single Entity**: A transaction references exactly one entity.
- **Verification Gate**: Categories `ARCHITECTURE`, `FEATURE`, `BUGFIX`, and `INFRA` require `verification_status` and `verification_basis` fields at commit time. The `note` command bypasses this but restricts itself to low-risk categories (`DOCS`, `CHORE`, `TOOLING`, `REFACTOR`).
- **Immutable Ledger**: `ledger commit` transitions `transactions` status to `COMMITTED` and writes an immutable record to `ledger_entries` (synced to `ledger_fts`).

### 3.4 Data Access & Error Handling
- Use the `uuid` crate (v4) for generating `tx_id`.
- Use the typed `LedgerError` (via `thiserror` + `miette::Diagnostic`) from `src/ledger/error.rs`.
- Do not use `unwrap`/`expect` in production paths.

## 4. Testing & Validation (TDD)
- Tests go into `tests/ledger_lifecycle.rs`.
- Must test full round trips: `start -> commit`, `start -> rollback`, `atomic`.
- Verify `note` mode restricts categories properly.
- Verify `resume` correctly fetches the last pending transaction.
- Verify fuzzy UUID matching logic.
- Verify conflict detection when starting a transaction on an entity that already has a `PENDING` transaction.
- Verify missing file (ghost commit guard) warns during commit.
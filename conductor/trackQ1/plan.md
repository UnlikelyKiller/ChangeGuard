## Plan: Track Q1 - Signature Integrity Fix
### Phase 1: Struct and Data Flow Extensions
- [x] Task 1.1: Add `pub committed_at: Option<String>` to `CommitRequest` struct in `src/ledger/types.rs`.
- [x] Task 1.2: Add `pub committed_at: Option<String>` to `PendingHookTx` in both `src/commands/hook_commit_msg.rs` and `src/commands/hook_post_commit.rs`.

### Phase 2: Implementation of Preserved Timestamps
- [x] Task 2.1: In `src/commands/hook_commit_msg.rs`, update the initialization of `PendingHookTx` to include `committed_at: Some(committed_at)`.
- [x] Task 2.2: In `src/commands/hook_post_commit.rs`, update the initialization of `CommitRequest` to map `committed_at: pending.committed_at`.
- [x] Task 2.3: In `src/ledger/transaction.rs`, modify `commit_change` to use `req.committed_at` if provided: `let now = req.committed_at.unwrap_or_else(|| Utc::now().to_rfc3339());`.

### Phase 3: Testing & Verification
- [x] Task 3.1: Add a test in `tests/ledger_crypto.rs` (or equivalent test file) demonstrating that passing a predefined `committed_at` in `CommitRequest` ensures `commit_change` persists that exact timestamp, thus maintaining signature validity.

# Track Q1: Signature & Ledger Integrity
## Objective
Investigate and resolve the root cause of `INVALID` signatures shown when running `verify --signatures`. Specifically, mathematical validation of signatures currently fails for 10 out of 28 ledger entries.

## Root Cause Analysis
The divergence point occurs between the `commit-msg` git hook and the `post-commit` git hook:
1. In `src/commands/hook_commit_msg.rs`, a new ledger entry is cryptographically signed using the timestamp `committed_at = chrono::Utc::now().to_rfc3339()`. The signature and public key are saved to a sidecar file (`PendingHookTx`). The timestamp `committed_at` used for the signature is **NOT** saved in this sidecar.
2. In `src/commands/hook_post_commit.rs`, the sidecar is read and a `CommitRequest` is built. Since `CommitRequest` does not have a `committed_at` field, it drops the concept of time.
3. In `src/ledger/transaction.rs`, the `commit_change()` function is called. It recalculates the timestamp via `let now = Utc::now().to_rfc3339();` and saves this *new* timestamp to the SQLite database.
4. When `verify --signatures` runs, it rebuilds the exact string payload using the database's `committed_at` value. Since this differs from the timestamp used during the pre-commit hook by a few milliseconds/seconds, the signature validation mathematically fails. 

Manually committed entries via `changeguard ledger commit` execute entirely within `commit_change()`, meaning the signature is generated and saved using the same `now` variable simultaneously, resulting in VALID signatures. The 10 INVALID entries correspond directly to commits generated via git hooks.

## Requirements
1. **Timestamp Preservation**: The exact timestamp used to generate the cryptographic signature must be passed down to the ledger persistence layer.
2. **Backward Compatibility**: Existing APIs (e.g., `CommitRequest::default()`) should not break. The new `committed_at` field on requests should be optional. If absent, fallback to `Utc::now().to_rfc3339()`.

## API Contracts / Struct Changes
1. **`src/commands/hook_commit_msg.rs` & `src/commands/hook_post_commit.rs`**
   - Add `pub committed_at: Option<String>` to `PendingHookTx`.
2. **`src/ledger/types.rs`**
   - Add `pub committed_at: Option<String>` to `CommitRequest` (existing derives will naturally handle the `Option`).
3. **`src/ledger/transaction.rs`**
   - Update `TransactionManager::commit_change` to extract `committed_at` from `req` or generate a new one:
     `let now = req.committed_at.unwrap_or_else(|| Utc::now().to_rfc3339());`

## Testing Strategy
1. Modify `tests/ledger_crypto.rs` to verify that `commit_change` correctly adopts an explicitly passed `committed_at` timestamp in the `CommitRequest`.
2. Ensure existing tests using `CommitRequest { ..Default::default() }` still pass.

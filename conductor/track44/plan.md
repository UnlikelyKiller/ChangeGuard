# Implementation Plan - Track 44: Atomic Commit-Commit

## Goal
Enable `changeguard ledger commit --with-git` to atomically close a ledger transaction and create a corresponding git commit.

## Proposed Changes

### 1. CLI Definition [src/cli.rs]
Add to `LedgerCommands::Commit`:
- `--with-git` (bool flag): After ledger commit, invoke `git commit`.
- `--git-message` (optional string): Override auto-generated git commit message.
- `--no-signoff` (bool flag, optional): Skip automatic `Signed-off-by` line.

### 2. Git Commit Wrapper [src/git/commit.rs] [NEW]
- `pub fn commit(message: &str, signoff: bool) -> Result<()>`
- Build argv: `["git", "commit", "-m", message]` (plus `"--signoff"` if enabled).
- Execute via `std::process::Command` with `ProcessPolicy` timeout.
- Parse stdout/stderr for common failure modes (nothing to commit, pre-commit hook failure, merge conflict) and map to descriptive `miette` errors.

### 3. Ledger Commit Handler Update [src/commands/ledger.rs]
- Update `execute_ledger_commit` signature to accept `with_git: bool`, `git_message: Option<String>`, `no_signoff: bool`.
- After successful `ledger commit`:
  - If `with_git`:
    - Build default message: `[{category}] {summary}\n\nLedger: {tx_id}`.
    - Use `--git-message` if provided.
    - Call `git::commit::commit(...)`.
    - On success: print confirmation.
    - On failure: print warning with exact command to retry manually.

### 4. Error Handling
- Git failure must not abort or roll back the already-successful ledger commit.
- Warning format: `Git commit failed: {reason}. Ledger commit is complete. Retry with: git commit -m "..."`.

### 5. Tests
- `tests/ledger_git_commit.rs`:
  - `test_with_git_success`: Mock git executable that succeeds. Assert both ledger and git are called.
  - `test_with_git_failure_preserves_ledger`: Mock git that fails. Assert ledger is still committed and warning is emitted.
  - `test_git_message_override`: Assert custom message is passed to git.
  - `test_default_message_format`: Assert generated message contains category, summary, and tx_id.

## Verification Plan

### Automated Tests
- `cargo test --test ledger_git_commit`
- `cargo test --workspace`

### Manual Verification
- Create a PENDING transaction, stage a file, run `changeguard ledger commit <tx-id> --with-git`, verify `git log` shows the commit.

## Definition of Done (DoD)
- [ ] **CLI Flags**: `--with-git` and `--git-message` are available on `ledger commit`.
- [ ] **Git Wrapper**: `src/git/commit.rs` exists with safe argv-based invocation.
- [ ] **Atomic Behavior**: Ledger commit succeeds even if git commit fails; user is warned.
- [ ] **Message Quality**: Default git message includes ledger category, summary, and tx_id.
- [ ] **Test Coverage**: Mock-based integration tests cover success, failure, and override paths.
- [ ] **Zero Regression**: Existing ledger tests pass.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

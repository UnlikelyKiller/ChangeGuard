# Implementation Plan - Track 44: Atomic Commit-Commit

## Goal
Enable `changeguard ledger commit --with-git` to atomically close a ledger transaction and create a corresponding git commit.

## Proposed Changes

### 1. CLI Definition [src/cli.rs]
Add to `LedgerCommands::Commit`:
- `--with-git` (bool flag): After ledger commit, invoke `git commit`.
- `--git-message` (optional string): Override auto-generated git commit message.
- `--no-signoff` (bool flag, optional): Skip automatic `Signed-off-by` line.
- `--dry-run` (bool flag, optional): Print the git command without executing.

### 2. Git Commit Wrapper [src/git/commit.rs] [NEW]
- **Design rationale**: Shell out to the `git` binary (not `git2`) to preserve user hooks (`pre-commit`, `commit-msg`, `post-commit`), GPG signing, and `.gitconfig`.
- `pub fn commit(message: &str, signoff: bool) -> Result<()>`
- Build argv: `["git", "commit", "-m", message]` (plus `"--signoff"` if enabled).
- Execute via `std::process::Command` with `ProcessPolicy` timeout.
- `pub fn can_commit() -> Result<bool, GitStateError>` checks:
  - `git diff --cached --quiet` (exit 0 = no staged changes → return false).
  - `git rev-parse --git-path MERGE_HEAD` exists → `MergeInProgress`.
  - `git diff --name-only --diff-filter=U` non-empty → `ConflictsRemaining`.
- `GitCommitError` taxonomy maps stderr to actionable variants:
  ```rust
  pub enum GitCommitError {
      NothingToCommit,
      PreCommitHookFailed { exit_code: i32, stderr: String },
      MergeInProgress,
      ConflictsRemaining,
      GpgSigningFailed,
      Other { stderr: String },
  }
  ```
- Add `GIT_BINARY` env override for testability.

### 3. Ledger Commit Handler Update [src/commands/ledger.rs]
- Update `execute_ledger_commit` signature to accept `with_git: bool`, `git_message: Option<String>`, `no_signoff: bool`.
- After successful `ledger commit`:
  - If `with_git`:
    - Use config `git_commit_template` or fallback `[{category}] {summary}\n\nLedger: {tx_id}`.
    - Use `--git-message` if provided.
    - If `--dry-run`: print the git command and exit without invoking.
    - Call `git::commit::can_commit()` first; if false, print suggestion (stage files or use `--amend`).
    - Call `git::commit::commit(...)`.
    - On success: print confirmation.
    - On failure: print warning with exact command to retry manually. If failure is `NothingToCommit`, suggest `--amend`.

### 4. Error Handling
- Git failure must not abort or roll back the already-successful ledger commit.
- Warning format: `Git commit failed: {reason}. Ledger commit is complete. Retry with: git commit -m "..."`.
- If `can_commit()` returns false, emit a non-fatal suggestion before attempting the commit.

### 5. Tests
- `tests/ledger_git_commit.rs`:
  - `test_with_git_success`: Mock git executable that succeeds. Assert both ledger and git are called.
  - `test_with_git_failure_preserves_ledger`: Mock git that fails. Assert ledger is still committed and warning is emitted.
  - `test_git_message_override`: Assert custom message is passed to git.
  - `test_default_message_format`: Assert generated message contains category, summary, and tx_id.
  - `test_dry_run_prints_command`: Assert `--dry-run` prints git command without executing.
  - `test_nothing_to_commit_suggests_amend`: Mock git returning "nothing to commit." Assert `--amend` suggestion.
  - `test_merge_in_progress_blocked`: Mock git in merge state. Assert `can_commit()` returns false.

## Verification Plan

### Automated Tests
- `cargo test --test ledger_git_commit`
- `cargo test --workspace`

### Manual Verification
- Create a PENDING transaction, stage a file, run `changeguard ledger commit <tx-id> --with-git`, verify `git log` shows the commit.

## Definition of Done (DoD)
- [ ] **CLI Flags**: `--with-git`, `--git-message`, and `--dry-run` are available on `ledger commit`.
- [ ] **Git Wrapper**: `src/git/commit.rs` exists with safe argv-based invocation (shells out to binary for hook compatibility).
- [ ] **Git State Checks**: `can_commit()` detects no staged changes, merge state, and conflicts.
- [ ] **Error Taxonomy**: `GitCommitError` maps all common failure modes to actionable variants.
- [ ] **Atomic Behavior**: Ledger commit succeeds even if git commit fails; user is warned.
- [ ] **Message Quality**: Default git message uses configurable template; fallback includes category, summary, and tx_id.
- [ ] **Test Coverage**: Mock-based tests cover success, failure, override, dry-run, nothing-to-commit, and merge-in-progress.
- [ ] **Zero Regression**: Existing ledger tests pass.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

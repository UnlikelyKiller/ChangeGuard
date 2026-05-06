# Track 44: Atomic Commit-Commit

## Overview
Users currently must run `changeguard ledger commit` and `git commit` as two separate commands. This creates friction and can lead to ledger-to-git desynchronization (ledger committed but git forgotten, or vice versa). An atomic "commit-commit" command that performs both operations in a single invocation would reduce this friction and improve transactional provenance reliability.

## Objectives
- Add `--with-git` flag to `changeguard ledger commit`.
- When `--with-git` is present, after the ledger commit succeeds, automatically invoke `git commit` with a message derived from the ledger entry.
- If `git commit` fails, the ledger entry remains committed (it is the source of truth), but a clear warning is emitted instructing the user to resolve the git side manually.
- Support optional `--git-message` to override the auto-generated git commit message.

## Architecture
- `src/cli.rs` — Add `--with-git` and `--git-message` flags to `LedgerCommands::Commit`.
- `src/commands/ledger.rs` — Update `execute_ledger_commit` to accept new flags.
- `src/git/commit.rs` [NEW] — Thin wrapper around `git commit` invocation.
  - `git_commit(message: &str, signoff: bool) -> Result<()>`
  - Uses `std::process::Command` (argv-style, not shell strings).
  - Returns descriptive error on failure.
- `src/platform/process_policy.rs` — Reuse existing `ProcessPolicy` for timeout and execution safety.

## Success Criteria
- `changeguard ledger commit <tx-id> --with-git` successfully commits both ledger and git.
- Generated git message includes ledger summary and category for traceability.
- Git failure does not roll back the ledger commit; instead it prints a actionable warning.
- Works on Windows (PowerShell) and Unix without shell string injection.
- New unit and integration tests cover success and failure paths.

## Testing Strategy
- **Red commit**: Write tests that mock `git commit` invocation. Test success path, failure path, and message formatting.
- **Green commit**: Implement `--with-git` integration. Verify all tests pass.

# Track GF14: Ledger Command Group Split

## Objective

Split `src/commands/ledger.rs` (1,006 lines, 1,006 production — zero test lines) into command-group modules. The file contains 13 distinct `execute_ledger_*` handlers covering the full ledger CLI surface. While all handlers are ledger-domain, the file has no tests and has grown large enough that navigation and extension are impaired.

Note: This is the softest of the six GF candidates. The handlers are all in the same domain (ledger), unlike the clear cross-concern splits in GF9–GF13. The split is justified primarily by navigability and the absence of any tests: splitting by command group also creates natural homes for per-group unit tests.

## Evidence

- 1,006 lines, all production code, zero `#[cfg(test)]` lines
- 13 public `execute_*` functions covering: start, commit, rollback, atomic, resume (lifecycle); reconcile, adopt, gc, hook_repair (maintenance); register_rule, register_validator (registration); status, export_provenance (reporting)
- Related commands already have dedicated files: `ledger_adr.rs`, `ledger_audit.rs`, `ledger_graph.rs`, `ledger_register.rs`, `ledger_search.rs`, `ledger_stack.rs` — this file holds the core lifecycle/maintenance handlers that were never split off
- `execute_ledger_status` is the largest single handler (likely 150-200 lines); the rest are 30–80 lines each

## Scope

Facade pattern: keep `src/commands/ledger.rs` as the facade file and add a sibling `src/commands/ledger/` directory (GF8 `dead_code.rs` pattern). `mod lifecycle;` declared inside `ledger.rs` resolves to `ledger/lifecycle.rs`. No rename to `ledger/mod.rs` at any point — having both files is E0761.

| Module | Assigned handlers |
|---|---|
| `ledger.rs` (facade) | `mod` declarations + `pub use` re-exports only |
| `ledger/lifecycle.rs` | `execute_ledger_start`, `execute_ledger_commit`, `LedgerCommitGitOptions`, `execute_git_commit`, `display_git_commit_command`, `execute_ledger_rollback`, `execute_ledger_atomic`, `execute_ledger_resume`, `resolve_start_category` |
| `ledger/maintenance.rs` | `execute_ledger_gc`, `execute_ledger_hook_repair`, `execute_ledger_reconcile`, `execute_ledger_adopt` |
| `ledger/registration.rs` | `execute_ledger_register_rule`, `execute_ledger_register_validator` |
| `ledger/reporting.rs` | `execute_ledger_status`, `execute_ledger_export_provenance`, `write_ledger_graph_edges` |

All 13 public functions remain reachable at their existing import paths via facade re-exports.

Private helpers stay in the module where they are used.

## Non-Goals

- No behavior changes to any handler.
- No new ledger commands.
- No changes to the CLI argument types in `src/cli/args.rs`.
- No touching `.changeguard` state files.

## Implementation Notes

- `execute_ledger_commit` is the most complex handler — it calls `execute_git_commit` and `display_git_commit_command`, which move with it into `lifecycle.rs` and can stay private there.
- `write_ledger_graph_edges` is private, only called by `execute_ledger_export_provenance`. Both go to `reporting.rs`.
- `resolve_start_category` is private, only called by `execute_ledger_start`. It goes to `lifecycle.rs`.
- The existing separate files (`ledger_adr.rs`, `ledger_audit.rs`, etc.) are siblings in `src/commands/`, not part of `src/commands/ledger/`. They are unaffected by this track.
- No changes to `src/commands/mod.rs` are needed — `mod ledger;` continues to resolve to `ledger.rs`.
- **Testing reality check**: the `execute_*` handlers open repository state from the working directory and have console side effects — they are not unit-testable without cwd manipulation, which breaks parallel test isolation. Unit tests target the pure helpers (`resolve_start_category`, `display_git_commit_command`); handler behavior is already covered by `tests/integration`. Do not write cwd-dependent unit tests.

## Verification Strategy

Targeted (run after each module move):
- `cargo check --all-targets --all-features`

Final:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/commands/ledger.rs` contains only `mod` declarations and `pub use` re-exports.
- Each command group (lifecycle, maintenance, registration, reporting) lives in its own module under `src/commands/ledger/`.
- All 13 `execute_ledger_*` functions remain importable from `crate::commands::ledger`.
- Pure helpers (`resolve_start_category`, `display_git_commit_command`) gain unit tests; handler coverage is confirmed via the existing integration suite.
- Full verification and reinstall pass.
- Ledger transaction committed; `changeguard ledger status --compact` shows `0 pending, 0 unaudited drift`.

## Risks

- `execute_ledger_status` is the largest handler (~200 lines) and references private types from `src/ledger/` — ensure all required imports move with it to `reporting.rs`.
- Handlers are not unit-testable without cwd manipulation; resist the temptation to add such tests. The integration suite (`tests/integration`) is the behavioral safety net for this track — run it after every phase, not just at the end.

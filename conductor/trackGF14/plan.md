# Track GF14 Plan: Ledger Command Group Split

## Phase 0: Baseline and Guardrails

- [ ] Confirm ledger state: `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF14 --category REFACTOR --message "Ledger command handlers split by command group"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `cargo check --all-targets --all-features` and confirm clean.
- [ ] Run `cargo nextest run --test integration` and record the baseline — the integration suite is this track's behavioral safety net since the file has zero unit tests.
- [ ] Inventory all 13 `pub fn` handlers and assign each to a command group per the spec table.
- [ ] Identify private helpers and assign each to the module containing its only caller.

Definition of done: Handler mapping complete; integration baseline green; ledger open.

## Phase 1: Directory Scaffold

- [ ] Create `src/commands/ledger/` directory. `ledger.rs` stays as the facade — do NOT rename to `ledger/mod.rs` (E0761 risk; GF8 facade-file pattern is the target shape). `src/commands/mod.rs` needs no changes.
- [ ] No empty stub files — each group module is created in the step that moves its handlers.

Definition of done: Directory exists; clean compile.

## Phase 2: Handler Moves

Move one group at a time. After each: `cargo check --all-targets --all-features`.

- [ ] Create `ledger/registration.rs`; move `execute_ledger_register_rule`, `execute_ledger_register_validator` (mark `pub(super)`).
  - Add `mod registration; pub use registration::{execute_ledger_register_rule, execute_ledger_register_validator};` to `ledger.rs`.
  - Run `cargo check`.
- [ ] Create `ledger/maintenance.rs`; move `execute_ledger_gc`, `execute_ledger_hook_repair`, `execute_ledger_reconcile`, `execute_ledger_adopt` (mark `pub(super)`).
  - Add `mod maintenance;` + re-exports to `ledger.rs`.
  - Run `cargo check`.
- [ ] Create `ledger/reporting.rs`; move `execute_ledger_status`, `execute_ledger_export_provenance` (mark `pub(super)`), and `write_ledger_graph_edges` (stays private).
  - Add `mod reporting;` + re-exports to `ledger.rs`.
  - Run `cargo check`.
- [ ] Create `ledger/lifecycle.rs`; move `execute_ledger_start`, `resolve_start_category` (private), `execute_ledger_commit`, `LedgerCommitGitOptions`, `execute_git_commit` (private), `display_git_commit_command` (private), `execute_ledger_rollback`, `execute_ledger_atomic`, `execute_ledger_resume`.
  - Add `mod lifecycle;` + re-exports for the public handlers and `LedgerCommitGitOptions` to `ledger.rs`.
  - Run `cargo check`.
- [ ] Confirm `ledger.rs` contains only `mod` declarations and `pub use` re-exports.
- [ ] Run `cargo nextest run --test integration` and confirm it matches the Phase 0 baseline.

Definition of done: All 13 handlers in their command-group modules; facade is pure; integration suite green.

## Phase 3: Unit Tests for Pure Helpers

Handlers open repository state from the working directory and are covered by the integration suite — do NOT write cwd-dependent unit tests for them. Unit tests target the pure helpers only:

- [ ] `lifecycle.rs`: test `resolve_start_category` with each valid category string and at least one invalid input.
- [ ] `lifecycle.rs`: test `display_git_commit_command` output formatting (with and without signoff).
- [ ] Run `cargo test commands::ledger` and confirm the new tests pass.

Definition of done: Pure helpers have unit tests; all pass.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit: `changeguard ledger commit <tx-id> --summary "Completed Track GF14: ledger command handlers split into groups" --reason "1,006-line zero-test file split into 4 command-group modules with pure-helper unit tests"`. If the git pre-commit hook removed the sidecar and `ledger status` still shows 1 pending after the git commit, run `ledger commit` again immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.
- [ ] Mark all tasks `- [x]` in this plan and set Status: Completed in `conductor/conductor.md`.

Definition of done: Full gates pass; installed binary matches source; ledger clean; conductor registry current.

## Plan: Restore Ledger Note

### Phase 1: CLI Definition & Dispatch
- [ ] Task 1.1: Add `LedgerCommands::Note` to `src/cli/args.rs` with `entity` (positional), `note` (positional, `required_unless_present = "message"`), and `message` (`short, long`). Ensure no deprecation strings are present.
- [ ] Task 1.2: Add `execute_ledger_note` to `src/commands/ledger/lifecycle.rs`. Ensure it resolves the `message` over `note` and calls `tx_mgr.atomic_change(..., category: Chore, reason: "Lightweight note", force: false)`.
- [ ] Task 1.3: Re-export `execute_ledger_note` in the ledger module (`src/commands/ledger.rs`).
- [ ] Task 1.4: Wire up `LedgerCommands::Note` in `src/cli/dispatch.rs` -> `dispatch_ledger()` to call `execute_ledger_note`.

### Phase 2: Testing & Validation
- [ ] Task 2.1: Add `test_note_success_positional` to `tests/integration/ledger_cli_parsing.rs` demonstrating `changeguard ledger note docs/file.md "my note"`.
- [ ] Task 2.2: Add `test_note_success_flag` to `tests/integration/ledger_cli_parsing.rs` demonstrating `changeguard ledger note docs/file.md --message "my note"`.
- [ ] Task 2.3: Add `test_note_failure_missing_both` to `tests/integration/ledger_cli_parsing.rs` ensuring clap errors when neither is provided.
- [ ] Task 2.4: Ensure `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` pass.

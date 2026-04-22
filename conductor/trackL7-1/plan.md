# Plan: Track L7-1 Production Polish

### Phase 1: Unit Level Polish (Formatting & Timestamps)
- [ ] Task 1.1: Create a utility module (e.g., `src/ledger/ui.rs` or extend `src/output/`) for icon mapping and color-coding logic.
- [ ] Task 1.2: Write unit tests for icon mapping (ensuring correct icons for Category, ChangeType, Status).
- [ ] Task 1.3: Implement relative timestamp formatting (e.g., "2 hours ago") and write unit tests for it.
- [ ] Task 1.4: Refactor `LedgerError` in `src/ledger/error.rs` to include `#[help(...)]` attributes for actionable diagnostics using `miette`.

### Phase 2: CLI Command UI & Help Polish
- [ ] Task 2.1: Update `src/commands/ledger_status.rs` to use consistent table formatting, color-coded icons, and relative timestamps. Verify `--compact` mode functionality.
- [ ] Task 2.2: Update `src/commands/ledger_search.rs` to share the same table formatting and aesthetic as `status`.
- [ ] Task 2.3: Update `src/commands/ledger_audit.rs` to align with the polished aesthetic.
- [ ] Task 2.4: Audit all ledger subcommands (`src/commands/ledger*.rs`) and add descriptive `about`, `long_about` (with examples), and `help` strings to clap structs.

### Phase 3: Documentation Updates
- [ ] Task 3.1: Update `README.md` with a comprehensive overview of Ledger features, commands, and examples.
- [ ] Task 3.2: Update `.agents/skills/changeguard/skill.md` with complete documentation for all ledger commands, tailored for AI agent usage.

### Phase 4: Final Review
- [ ] Task 4.1: Run `cargo test` to ensure all tests pass (especially new UI utilities).
- [ ] Task 4.2: Verify visual consistency by invoking help commands and dummy status/search data.
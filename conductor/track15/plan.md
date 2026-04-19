## Plan: Track 15 — Gemini Modes, Output Module, Git Classification

### Phase 1: Gemini Modes
- [ ] Task 15.1: Create `src/gemini/modes.rs`. Define `GeminiMode` enum with `Analyze`, `Suggest`, `ReviewPatch`.
- [ ] Task 15.2: Implement mode-specific system prompts and user prompt templates in `modes.rs`.
- [ ] Task 15.3: Add `--mode` flag to `ask` command in `src/cli.rs` with default `analyze`. Update `execute_ask` to accept mode.
- [ ] Task 15.4: Write unit tests verifying each mode produces distinct prompt content. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 2: Output Module
- [ ] Task 15.5: Create `src/output/mod.rs`, `src/output/human.rs`, `src/output/json.rs`, `src/output/table.rs`, `src/output/diagnostics.rs`. Register in `src/lib.rs`.
- [ ] Task 15.6: Extract scan output formatting from `commands/scan.rs` into `output::human::print_scan_summary`.
- [ ] Task 15.7: Extract impact output formatting from `commands/impact.rs` into `output::human::print_impact_summary`.
- [ ] Task 15.8: Extract doctor output formatting from `commands/doctor.rs` into `output::human::print_doctor_report`.
- [ ] Task 15.9: Extract verify output formatting from `commands/verify.rs` into `output::human::print_verify_result`.
- [ ] Task 15.10: Move `ui/mod.rs` helpers into `output/diagnostics.rs`. Update all references. Remove `src/ui/mod.rs` and `ui` from `lib.rs`.
- [ ] Task 15.11: Implement `output::json::format_json`. Implement `output::table` shared table builder. Implement `output::diagnostics` shared diagnostic helpers.
- [ ] Task 15.12: Write unit tests for each output formatter. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Git Classification Fix
- [ ] Task 15.13: Audit gix status API to understand `Purpose` variants and how they distinguish added/modified/deleted/renamed.
- [ ] Task 15.14: Rewrite `src/git/classify.rs` to correctly map gix status entries to `ChangeType` variants. Ensure `Added`, `Deleted`, `Renamed` are emitted where appropriate.
- [ ] Task 15.15: Ensure `is_staged` correctly reflects index vs worktree state.
- [ ] Task 15.16: Write integration tests in `tests/cli_scan.rs` creating repos with added, deleted, and renamed files. Assert correct `ChangeType` values.
- [ ] Task 15.17: Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 4: Final Verification
- [ ] Task 15.18: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 15.19: Full suite `cargo test -j 1 -- --test-threads=1`.
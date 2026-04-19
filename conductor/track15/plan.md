## Plan: Track 15 — Gemini Modes, Output Module, Git Classification

### Phase 1: Gemini Modes
- [ ] Task 15.1: Create `src/gemini/modes.rs`. Define `GeminiMode` enum with `Analyze`, `Suggest`, `ReviewPatch`. Implement `Display` for CLI parsing.
- [ ] Task 15.2: Implement mode-specific system prompts and user prompt templates in `modes.rs`. For `ReviewPatch`, define the diff-inclusion logic.
- [ ] Task 15.3: Add `--mode` flag to `ask` command in `src/cli.rs` with default `analyze` and possible values. Update `execute_ask` to accept mode.
- [ ] Task 15.4: Implement `ReviewPatch` diff extraction: run `git diff HEAD` (or `git diff --cached`), include in prompt. Fall back to `Analyze` on clean tree with user note.
- [ ] Task 15.5: Write unit tests verifying each mode produces distinct prompt content. Test `ReviewPatch` fallback. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 2: Output Module
- [ ] Task 15.6: Create `src/output/mod.rs`, `src/output/human.rs`, `src/output/json.rs`, `src/output/diagnostics.rs`. Register in `src/lib.rs`.
- [ ] Task 15.7: Extract scan output formatting from `commands/scan.rs` into `output::human::print_scan_summary`.
- [ ] Task 15.8: Extract impact output formatting from `commands/impact.rs` into `output::human::print_impact_summary`.
- [ ] Task 15.9: Extract doctor output formatting from `commands/doctor.rs` into `output::human::print_doctor_report`.
- [ ] Task 15.10: Extract verify output formatting from `commands/verify.rs` into `output::human::print_verify_result`. Add `print_verify_plan` for plan display.
- [ ] Task 15.11: Move `ui/mod.rs` helpers into `output/diagnostics.rs`. Update ALL references across the codebase. Remove `src/ui/mod.rs` and `ui` from `lib.rs`. This must be done atomically — all changes in one commit.
- [ ] Task 15.12: Implement `output::json::format_json` (stub: basic serde_json pretty-print wrapper). Implement `output::diagnostics` shared diagnostic helpers (error banner, warning banner).
- [ ] Task 15.13: Write unit tests for each human output formatter (verify structure/content). Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Git Classification Fix
- [ ] Task 15.14: Audit gix status API: read gix source/docs to understand how `index_worktree_rewrites` entries distinguish added/modified/deleted/renamed. Map the actual gix `Status`/`Purpose`/`Stage` types to `ChangeType`.
- [ ] Task 15.15: Rewrite `src/git/classify.rs` to correctly map gix status entries to `ChangeType` variants. Ensure `Added`, `Deleted`, `Renamed` are emitted. Handle files with both staged and unstaged changes (two entries).
- [ ] Task 15.16: Recreate `src/git/diff.rs` with `pub fn get_diff_summary(repo: &gix::Repository, path: &Path) -> Option<String>` returning first 50 lines of diff for a file. Register in `src/git/mod.rs`.
- [ ] Task 15.17: Ensure `is_staged` correctly reflects index vs worktree state for all change types.
- [ ] Task 15.18: Write integration tests in `tests/cli_scan.rs` creating repos with added, deleted, renamed, and mixed staged/unstaged files. Assert correct `ChangeType` and `is_staged` values.
- [ ] Task 15.19: Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 4: Final Verification
- [ ] Task 15.20: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 15.21: Full suite `cargo test -j 1 -- --test-threads=1`.
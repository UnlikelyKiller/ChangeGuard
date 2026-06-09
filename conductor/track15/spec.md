# Specification: Track 15 — Gemini Modes, Output Module, Git Classification

## Overview
Address audit items 4, 5, 7, 8: implement Gemini prompt modes, extract the output formatting module, and fix git change classification.

## 1. Gemini Modes (`src/gemini/modes.rs`)
**Priority: HIGH** — Plan Phase 13 specifies mode support.

### Mode Enum
```rust
pub enum GeminiMode {
    Analyze,      // General change analysis (default)
    Suggest,      // Suggest targeted verification
    ReviewPatch,  // Review a specific patch/diff
}
```

### Mode-Specific Prompts
- Each mode produces a different system prompt persona and user prompt template
- `Analyze`: focused on blast radius and risk explanation (current behavior)
- `Suggest`: focused on actionable verification recommendations (what to run, what to check)
- `ReviewPatch`: focused on code review of the changes themselves (requires diff content)

### ReviewPatch Mode Details
- `ReviewPatch` needs a diff/patch input. When this mode is selected:
  - The `ask` command runs `git diff HEAD` (or `git diff --cached` if no unstaged changes) to get the actual diff
  - The diff is included in the user prompt alongside the impact packet
  - If no diff is available (clean tree), fall back to `Analyze` mode with a note to the user

### Integration
- Add `--mode` flag to `ask` command with default `analyze`, possible values: `analyze`, `suggest`, `review-patch`
- `commands/ask.rs`: pass mode to `build_system_prompt` and `build_user_prompt`
- Register `modes` module in `src/gemini/mod.rs`

## 2. Output Module (`src/output/`)
**Priority: HIGH** — Extract formatting from command handlers per SRP.

### `src/output/mod.rs`
- Public module root, re-exports formatters

### `src/output/human.rs`
- Move all `println!` + `owo_colors` formatting from command handlers into dedicated functions:
  - `pub fn print_scan_summary(snapshot: &RepoSnapshot)`
  - `pub fn print_impact_summary(packet: &ImpactPacket)`
  - `pub fn print_doctor_report(results: &DoctorReport)`
  - `pub fn print_verify_result(result: &ExecutionResult)`
  - `pub fn print_verify_plan(plan: &VerificationPlan)`
- Each function owns its own `comfy_table` configuration and color choices

### `src/output/json.rs`
- `pub fn format_json<T: Serialize>(value: &T) -> Result<String>` — pretty-printed JSON
- Enables future `--format json` CLI flag
- **Stub implementation only**: just the function signature and basic serde_json call. Full integration with `--format json` flag is deferred (YAGNI until a consumer exists).

### `src/output/diagnostics.rs`
- Move `ui/mod.rs` helpers here: `print_header`, `success_marker`, `failure_marker`, `warning_marker`, `info_marker`
- Add shared diagnostic formatting: error banners, warning banners, section separators
- This replaces `src/ui/mod.rs` entirely

### No `table.rs` (YAGNI)
- The `table.rs` sub-module is over-engineering. Table configuration is naturally per-command (different headers, column counts). Extracting a shared builder adds abstraction without benefit. Keep table creation inline in `human.rs` functions.

### Integration
- Replace direct `println!` in all command handlers with calls to `output::human::*`
- Replace `use crate::ui::*` with `use crate::output::diagnostics::*`
- **Remove `src/ui/mod.rs`** and `ui` from `src/lib.rs` — this is the only breaking change. All references must be updated in the same commit.
- Register `output` module in `src/lib.rs`

## 3. Git Classification Fix (`src/git/classify.rs`)
**Priority: HIGH** — Currently all changes are emitted as `Modified`.

### Required Fixes
The current `classify_status` function only handles two `Purpose` variants and maps both to `Modified`. Must correctly distinguish:

- **Added**: File exists in worktree/index but not in HEAD tree. gix reports this as `TreeIndex` with `entry_status` indicating a new entry (the file has no previous version in the tree).
- **Deleted**: File exists in HEAD tree but not in worktree/index. Detected by checking if the entry represents a deletion in the status iteration.
- **Renamed**: gix reports rename entries when `index_worktree_rewrites` is enabled (which it is in current code). Map these to `ChangeType::Renamed { old_path }`.
- **Modified**: Default for files that exist in both tree and index/worktree with different content.

### `is_staged` Correctness
- `is_staged = true` when the change exists in the index (staged but not yet committed)
- `is_staged = false` when the change is only in the worktree (unstaged)
- Map `TreeIndex` entries (index vs tree) to `is_staged: true`
- Map `IndexWorktree` entries (worktree vs index) to `is_staged: false`
- A file can appear in BOTH if it has staged AND unstaged changes — this should produce two `FileChange` entries

### Approach
- Examine gix status API entry types to understand how to distinguish added vs modified vs deleted
- The `index_worktree_rewrites` iterator provides entries with rename detection
- Use the gix `Status` enum variants (which include `EntryToAdd`, `EntryToDelete`, etc.) or check the underlying diff to determine actual change type
- Add `src/git/diff.rs` back with a `pub fn get_diff_summary(repo, path) -> Option<String>` function for line-level diff summaries (addresses audit item 8)

## Verification
- Unit tests for each Gemini mode's prompt output (verify distinct content per mode)
- Unit test for `ReviewPatch` mode fallback to `Analyze` on clean tree
- Unit tests for output formatters verifying consistent structure (human formatters)
- Integration tests for git classification: create repos with added, modified, deleted, renamed files and verify correct `ChangeType` assignment
- Integration test for `is_staged` correctness: staged-only, unstaged-only, and both
- `cargo test -j 1 -- --test-threads=1`
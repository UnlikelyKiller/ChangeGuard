# Specification: Track 15 — Gemini Modes, Output Module, Git Classification

## Overview
Address audit items 4, 5, 7, 8: implement Gemini prompt modes, extract the output formatting module, and fix git change classification.

## 1. Gemini Modes (`src/gemini/modes.rs`)
**Priority: HIGH** — Plan Phase 13 specifies mode support.

### Mode Enum
```rust
pub enum GeminiMode {
    Analyze,      // General change analysis
    Suggest,      // Suggest targeted verification
    ReviewPatch,  // Review a specific patch/diff
}
```

### Mode-Specific Prompts
- Each mode produces a different system prompt persona and user prompt template
- `Analyze`: focused on blast radius and risk explanation
- `Suggest`: focused on actionable verification recommendations
- `ReviewPatch`: focused on code review of the changes themselves

### Integration
- Add `--mode` flag to `ask` command with default `Analyze`
- `commands/ask.rs`: pass mode to `build_system_prompt` and `build_user_prompt`
- Register `modes` module in `src/gemini/mod.rs`

## 2. Output Module (`src/output/`)
**Priority: HIGH** — Extract formatting from command handlers per SRP.

### `src/output/mod.rs`
- Public module root, re-exports formatters

### `src/output/human.rs`
- Move all `println!` + `owo_colors` formatting from `commands/scan.rs`, `commands/impact.rs`, `commands/doctor.rs`, `commands/verify.rs` into dedicated functions:
  - `pub fn print_scan_summary(snapshot: &RepoSnapshot)`
  - `pub fn print_impact_summary(packet: &ImpactPacket)`
  - `pub fn print_doctor_report(results: &DoctorReport)`
  - `pub fn print_verify_result(result: &ExecutionResult)`
  - `pub fn print_verify_plan(plan: &VerificationPlan)`

### `src/output/json.rs`
- `pub fn format_json<T: Serialize>(value: &T) -> Result<String>` — pretty-printed JSON
- Enables future `--format json` CLI flag

### `src/output/table.rs`
- Extract shared `comfy_table` configuration into reusable table builder

### `src/output/diagnostics.rs`
- Extract shared diagnostic formatting (error banners, warning markers)
- Move `ui/mod.rs` helpers here; deprecate `ui/` in favor of `output/`

### Integration
- Replace direct `println!` in all command handlers with calls to `output::human::*`
- Register `output` module in `src/lib.rs`

## 3. Git Classification Fix (`src/git/classify.rs`)
**Priority: HIGH** — Currently all changes are emitted as `Modified`.

### Required Fixes
- Distinguish `Added` when a file exists in the worktree but not in the index/HEAD
- Distinguish `Deleted` when a file exists in the index/HEAD but not in the worktree
- Distinguish `Renamed` when gix reports a rename entry
- Properly set `is_staged` based on index vs worktree state

### Approach
- Use gix's status entries which provide `Purpose` variants that distinguish index/worktree changes
- Map each `Purpose` variant to the correct `ChangeType`
- Ensure the `is_staged` flag reflects whether the change is in the index (staged) vs worktree only (unstaged)

## Verification
- Unit tests for each Gemini mode's prompt output
- Unit tests for output formatters verifying consistent structure
- Integration tests for git classification: create repos with added, modified, deleted, renamed files and verify correct `ChangeType` assignment
- `cargo test -j 1 -- --test-threads=1`
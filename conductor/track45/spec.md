# Track 45: Interactive Fix Suggestions

## Overview
When `changeguard verify` fails — for example because UNAUDITED drift is detected, a verification command returns a non-zero exit code, or predicted tests fail — users must manually diagnose the problem and remember the correct `changeguard` subcommand to reconcile the issue. This track adds an "Interactive Fix Suggestions" layer to `verify` output that maps common failure patterns to copy-paste-ready `changeguard` commands.

## Objectives
- After `verify` completes, analyze the failure state and append a "Suggested Actions" section to human-readable output.
- Map specific failure patterns to actionable commands:
  - `UNAUDITED` drift detected → `changeguard ledger reconcile --all --reason "verify follow-up"`
  - `PENDING` transactions older than N hours → `changeguard ledger status` + `changeguard ledger commit` or `rollback`
  - Verification command failed (e.g., test failure) → `changeguard impact --summary` to re-assess
  - Predicted test degradation → `changeguard verify --explain` for rationale
- Keep suggestions concise, copy-paste friendly, and safe (never destructive without explicit flags).
- Include suggestions in JSON output under a `suggested_actions` array for programmatic consumers.

## Architecture
- `src/verify/suggestions.rs` [NEW] — Suggestion engine.
  - `generate_suggestions(results: &VerifyResults, ledger_status: &LedgerStatus) -> Vec<Suggestion>`
  - `Suggestion { id: String, description: String, command: String, severity: SuggestionSeverity }`
  - Output is **deterministically sorted** by `severity` descending, then `description` ascending.
  - All logic is stateless; no randomness, no timestamps in sorting keys.
  - `SuggestionSeverity` derives `PartialOrd`/`Ord` for stable ordering: `Info < Warning < ActionRequired`.
- `src/verify/runner.rs` — Update `execute_verify` to collect ledger status alongside verification results.
- `src/commands/verify.rs` — Render suggestions in human output (respecting `NO_COLOR` and `--color` global flag) and inject into JSON output.
- `src/verify/results.rs` — Add `#[serde(default)] suggested_actions: Vec<Suggestion>` to `VerificationReport` for backward-compatible schema evolution.

## Success Criteria
- Running `changeguard verify` on a repo with `UNAUDITED` drift prints a suggestion to run `ledger reconcile`.
- Running `changeguard verify` after a test command failure prints a suggestion to re-run `impact --summary`.
- Suggestions are deterministically sorted and included in `latest-verify.json` under `suggested_actions`.
- No suggestions are printed when verify passes cleanly and ledger is clean.
- `NO_COLOR` env var and `--color` global flag are respected in human suggestion output.
- New unit tests for every mapped failure pattern, plus property-based safety invariants (no `--force`, no empty commands, deterministic order).
- Health suggestions: Running `changeguard verify --health` on a clean pass with stale pending transactions still emits warnings.

## Testing Strategy
- **Red commit**: Write tests asserting that specific failure states produce expected suggestion strings.
- **Green commit**: Implement suggestion engine. Verify all tests pass.

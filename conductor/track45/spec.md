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
  - `Suggestion { description: String, command: String, severity: SuggestionSeverity }`
- `src/verify/runner.rs` — Update `execute_verify` to collect ledger status alongside verification results.
- `src/commands/verify.rs` — Render suggestions in human output and inject into JSON output.

## Success Criteria
- Running `changeguard verify` on a repo with `UNAUDITED` drift prints a suggestion to run `ledger reconcile`.
- Running `changeguard verify` after a test command failure prints a suggestion to re-run `impact --summary`.
- Suggestions are included in `latest-verify.json` under `suggested_actions`.
- No suggestions are printed when verify passes cleanly.
- New unit tests for every mapped failure pattern.

## Testing Strategy
- **Red commit**: Write tests asserting that specific failure states produce expected suggestion strings.
- **Green commit**: Implement suggestion engine. Verify all tests pass.

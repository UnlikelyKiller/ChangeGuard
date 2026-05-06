# Implementation Plan - Track 45: Interactive Fix Suggestions

## Goal
Make `changeguard verify` failures self-healing by surfacing copy-paste-ready `changeguard` commands that address the root cause.

## Proposed Changes

### 1. Suggestion Engine [src/verify/suggestions.rs] [NEW]
- Define `Suggestion` and `SuggestionSeverity` (Info, Warning, ActionRequired).
- Implement `generate_suggestions` with the following mappings:
  - **UNAUDITED drift** (`ledger_status.unaudited_count > 0`):
    - Action: `changeguard ledger reconcile --all --reason "verify follow-up"`
    - Severity: `ActionRequired`
  - **PENDING transactions > 24h old**:
    - Action: `changeguard ledger status` (to inspect), then `changeguard ledger commit <tx-id> --summary "..."` or `changeguard ledger rollback <tx-id> --reason "stale"`
    - Severity: `Warning`
  - **Verification step non-zero exit**:
    - Action: `changeguard impact --summary` (re-assess blast radius)
    - Severity: `Warning`
  - **Predicted test degradation** (`verify_results.predicted_degradation == true`):
    - Action: `changeguard verify --explain` (see rationale)
    - Severity: `Info`
  - **No impact report available** (packet missing):
    - Action: `changeguard scan --impact`
    - Severity: `ActionRequired`

### 2. Ledger Status Integration [src/verify/runner.rs]
- Extend `VerifyResults` struct to optionally carry `ledger_status: Option<LedgerStatus>`.
- In `execute_verify`, after running verification steps, query `ledger status` (via `StorageManager` or `ledger::db` directly) to get `unaudited_count` and pending transaction ages.
- Pass combined state to `generate_suggestions`.

### 3. Output Rendering [src/commands/verify.rs]
- Human output:
  - After the verify summary table, print a "Suggested Actions" section with color-coded severity icons.
  - Format: `{severity} {description}\n   → {command}`
- JSON output (`latest-verify.json`):
  - Add `suggested_actions: Vec<Suggestion>` to the verify report schema.

### 4. Safety Rules
- Suggestions must never include `--force` or destructive flags by default.
- If a suggestion requires a `tx_id`, print the command with a placeholder (`<tx-id>`) and instruct the user to look it up via `ledger status`.
- No suggestions = clean pass. Do not print "no suggestions needed" noise.

### 5. Tests
- `test_suggestion_unaudited_drift`: Simulate ledger with unaudited entries. Assert reconcile suggestion.
- `test_suggestion_stale_pending`: Simulate 48h pending transaction. Assert status + commit/rollback suggestion.
- `test_suggestion_verify_failure`: Simulate command exit code 1. Assert impact summary suggestion.
- `test_suggestion_predicted_degradation`: Simulate degraded prediction. Assert `--explain` suggestion.
- `test_no_suggestions_on_clean_pass`: Assert empty suggestions when verify passes and ledger is clean.
- `test_json_output_includes_suggestions`: Assert `latest-verify.json` has `suggested_actions` array.

## Verification Plan

### Automated Tests
- `cargo test` in `src/verify/suggestions.rs`.
- `cargo test --workspace`.

### Manual Verification
- Introduce UNAUDITED drift in a test repo, run `changeguard verify`, confirm suggestion appears.
- Run `changeguard verify --json` and inspect `latest-verify.json` for `suggested_actions`.

## Definition of Done (DoD)
- [ ] **Suggestion Engine**: `src/verify/suggestions.rs` exists with all five mapped patterns.
- [ ] **Human Output**: Suggestions render after verify summary with severity colors.
- [ ] **JSON Output**: `latest-verify.json` includes `suggested_actions`.
- [ ] **Safety**: No suggestion includes implicit destructive flags.
- [ ] **Test Coverage**: Unit tests for every pattern and the clean-pass case.
- [ ] **Zero Regression**: Existing verify tests pass.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

# Implementation Plan - Track 45: Interactive Fix Suggestions

## Goal
Make `changeguard verify` failures self-healing by surfacing copy-paste-ready `changeguard` commands that address the root cause.

## Proposed Changes

### 1. Suggestion Engine [src/verify/suggestions.rs] [NEW]
- Define `Suggestion` and `SuggestionSeverity` (Info, Warning, ActionRequired).
- `SuggestionSeverity` derives `PartialOrd`/`Ord` for deterministic sorting:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
  pub enum SuggestionSeverity {
      Info,
      Warning,
      ActionRequired,
  }
  ```
- `Suggestion` includes a stable `id` for deduplication:
  ```rust
  pub struct Suggestion {
      pub id: String,        // e.g., "unaudited-drift-reconcile"
      pub description: String,
      pub command: String,
      pub severity: SuggestionSeverity,
  }
  ```
- Implement `generate_suggestions` with the following mappings:
  - **UNAUDITED drift** (`ledger_status.unaudited_count > 0`):
    - ID: `unaudited-drift-reconcile`
    - Action: `changeguard ledger reconcile --all --reason "verify follow-up"`
    - Severity: `ActionRequired`
  - **PENDING transactions > 24h old**:
    - ID: `stale-pending-status`
    - Action: `changeguard ledger status` (to inspect), then `changeguard ledger commit [TX_ID] --summary "..."` or `changeguard ledger rollback [TX_ID] --reason "stale"`
    - Severity: `Warning`
  - **Verification step non-zero exit**:
    - ID: `verify-failure-impact`
    - Action: `changeguard impact --summary` (re-assess blast radius)
    - Severity: `Warning`
  - **Predicted test degradation** (`verify_results.predicted_degradation == true`):
    - ID: `predicted-degradation-explain`
    - Action: `changeguard verify --explain` (see rationale)
    - Severity: `Info`
  - **Prediction warnings present** (e.g., structural or semantic prediction degraded):
    - ID: `prediction-warnings-explain`
    - Action: `changeguard verify --explain` and `changeguard impact --summary`
    - Severity: `Info`
  - **No impact report available** (packet missing):
    - ID: `missing-impact-scan`
    - Action: `changeguard scan --impact`
    - Severity: `ActionRequired`
- **Property-based invariants** (enforced in tests):
  - No suggestion command contains `--force` unless explicitly mapped.
  - No suggestion has an empty `command` string.
  - Output is always sorted by `severity` descending, then `description` ascending.

### 2. Ledger Status Integration [src/verify/runner.rs]
- Extend `VerifyResults` struct to optionally carry `ledger_status: Option<LedgerStatus>`.
- In `execute_verify`, after running verification steps, query `ledger status` (via `StorageManager` or `ledger::db` directly) to get `unaudited_count` and pending transaction ages.
- Pass combined state to `generate_suggestions`.
- **Health suggestions entry point**:
  - Implement `generate_health_suggestions(ledger_status: &LedgerStatus) -> Vec<Suggestion>` that emits warnings for stale pending transactions even when verify passes.
  - Gate behind `--health` CLI flag on `verify` to avoid noise in standard usage.

### 3. Output Rendering [src/commands/verify.rs]
- Human output:
  - After the verify summary table, print a "Suggested Actions" section with color-coded severity icons.
  - Format: `{severity} {description}\n   → {command}`
  - **Respect `NO_COLOR` env var and `--color` global flag**: disable color if `NO_COLOR` is set or `--color=never`.
- JSON output (`latest-verify.json`):
  - Add `#[serde(default)] suggested_actions: Vec<Suggestion>` to `VerificationReport` for backward-compatible schema evolution.
  - Ensure `Suggestion` serializes deterministically (sorted order is already guaranteed by `generate_suggestions`).

### 4. Safety Rules
- Suggestions must never include `--force` or destructive flags by default.
- If a suggestion requires a `tx_id`, print the command with a placeholder (`<tx-id>`) and instruct the user to look it up via `ledger status`.
- No suggestions = clean pass. Do not print "no suggestions needed" noise.

### 5. Tests
- `test_suggestion_unaudited_drift`: Simulate ledger with unaudited entries. Assert reconcile suggestion.
- `test_suggestion_stale_pending`: Simulate 48h pending transaction. Assert status + commit/rollback suggestion.
- `test_suggestion_verify_failure`: Simulate command exit code 1. Assert impact summary suggestion.
- `test_suggestion_predicted_degradation`: Simulate degraded prediction. Assert `--explain` suggestion.
- `test_suggestion_prediction_warnings`: Simulate prediction warnings. Assert both `--explain` and `impact --summary`.
- `test_no_suggestions_on_clean_pass`: Assert empty suggestions when verify passes and ledger is clean.
- `test_json_output_includes_suggestions`: Assert `latest-verify.json` has `suggested_actions` array.
- `test_no_force_in_suggestions`: Property test over all mappings; assert no command contains `--force`.
- `test_no_empty_commands`: Property test; assert no suggestion has an empty command string.
- `test_deterministic_sorting`: Generate suggestions in random input order; assert output is always sorted.
- `test_health_suggestions_on_clean_pass`: Run with `--health` flag and stale pending tx. Assert `Warning` is emitted.

## Verification Plan

### Automated Tests
- `cargo test` in `src/verify/suggestions.rs`.
- `cargo test --workspace`.

### Manual Verification
- Introduce UNAUDITED drift in a test repo, run `changeguard verify`, confirm suggestion appears.
- Run `changeguard verify --json` and inspect `latest-verify.json` for `suggested_actions`.

## Definition of Done (DoD)
- [ ] **Suggestion Engine**: `src/verify/suggestions.rs` exists with all five mapped patterns plus prediction-warnings mapping.
- [ ] **Deterministic Output**: Suggestions are always sorted by severity descending, description ascending.
- [ ] **Deduplication IDs**: Every `Suggestion` has a stable `id` for future suppression.
- [ ] **Human Output**: Suggestions render after verify summary with severity colors; respects `NO_COLOR` and `--color`.
- [ ] **JSON Output**: `latest-verify.json` includes `suggested_actions` with backward-compatible `serde(default)`.
- [ ] **Health Suggestions**: `--health` flag emits warnings on clean verify passes with stale pending transactions.
- [ ] **Safety**: No suggestion includes implicit destructive flags; property tests enforce this.
- [ ] **Test Coverage**: Unit tests for every pattern, clean-pass, property-based invariants, and deterministic sorting.
- [ ] **Zero Regression**: Existing verify tests pass.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

# Specification: Track M4-1 — Test Outcome Recording & Diff Embedding

## Objective
Build the data collection side of semantic test prediction: embed the current diff after each `verify` run and store test outcomes (pass/fail) linked to that embedding. This builds the history that M4-2's predictor queries.

## Components

### 1. `TestOutcome` Enum (`src/verify/semantic_predictor.rs`)

```rust
pub enum TestOutcome {
    Pass,
    Fail,
    Skip,
}

impl TestOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestOutcome::Pass => "pass",
            TestOutcome::Fail => "fail",
            TestOutcome::Skip => "skip",
        }
    }
}
```

### 2. Diff Text Construction (`src/verify/semantic_predictor.rs`)

```rust
pub fn build_diff_text(packet: &ImpactPacket) -> String
```

- Concatenate changed file paths + changed symbol names from the packet, space-separated
- Cap at first 200 items to bound length
- Returns empty string if packet has no changes

### 3. Outcome Recording (`src/verify/semantic_predictor.rs`)

```rust
pub fn record_test_outcomes(
    config: &Config,
    conn: &Connection,
    packet: &ImpactPacket,
    outcomes: &[(String, TestOutcome)],
) -> Result<()>
```

For each `verify` run:
1. Build diff text from packet via `build_diff_text()`
2. Call `embed_and_store` with `entity_type = "diff"`, `entity_id = blake3(diff_text)`
3. If embedding succeeds (returns `Ok(true)` or `Ok(false)`), retrieve the embedding row `id`
4. INSERT a row into `test_outcome_history` for each `(test_file, outcome)` pair, referencing `diff_embedding_id`
5. If `config.local_model.base_url` is empty, skip entirely without error (no rows inserted)

### 4. Hook into `execute_verify()` (`src/commands/verify.rs`)

After the verification runner completes:
1. Infer outcome per test file: exit code 0 = `Pass`, non-zero = `Fail`
2. Track test file name from `VerificationPlan` step
3. Call `record_test_outcomes` with the collected outcomes
4. Wrap the call so that recording failure never fails the verify command (log `WARN`, continue)

## Test Specifications

| Test | Assertion |
|---|---|
| `build_diff_text` 3 files + 5 symbols | Contains all paths and symbol names |
| `build_diff_text` empty packet | Returns empty string |
| `record_test_outcomes` 3 outcomes | Inserts 3 rows in `test_outcome_history` |
| Recording same diff twice | Embedding reused (same `diff_embedding_id`), outcomes accumulate |
| `base_url` empty | No rows inserted, no error |
| `execute_verify` integration | `test_outcome_history` has one new row after verify |
| Outcome recording failure | Does not propagate as error from `execute_verify` |
| `TestOutcome::as_str` each variant | Returns correct DB string |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **Non-blocking**: Outcome recording failure must never abort `verify`.
- **Test isolation**: Use `tempfile::tempdir()` for SQLite in tests.
- **No secrets**: Diff text is file paths + symbol names only; run through existing sanitizer before embedding.
- **CI safety**: When `base_url` is empty, no HTTP calls are made.

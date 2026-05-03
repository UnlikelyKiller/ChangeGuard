# Specification: Track M4-2 — Semantic Predictor & Score Blending

## Objective
Implement the semantic prediction layer: query past diff-test-outcome history for tests that historically failed on changes similar to the current one, blend those scores with the existing rule-based predictor, and surface the rationale via `--explain`.

## Components

### 1. Semantic Score Computation (`src/verify/semantic_predictor.rs`)

```rust
pub fn compute_semantic_scores(
    conn: &Connection,
    query_vec: &[f32],
    model_name: &str,
) -> Result<HashMap<String, f32>>
```

Algorithm:
1. Load all `"diff"` entity embeddings from `test_outcome_history` joined to `embeddings`
2. For each stored diff embedding, compute `cosine_sim(query_vec, stored_vec)`
3. Take top-30 most similar stored diffs by cosine score
4. For each `test_file` appearing in those 30 diffs' history rows:
   - Compute `weighted_score = sum(similarity * (fail_count / total_count))` across all matching history rows
5. Normalize all scores to [0.0, 1.0] by dividing by the max score (if max > 0)
6. Return as `HashMap<test_file, semantic_score>`

Cold start behavior:
- If fewer than 5 total history rows exist across all diffs: return empty `HashMap`
- Print: `Semantic prediction: warming up ({count}/50 history records)`
- If 50+ history rows: no warming up message

### 2. Verify Config Extension (`src/config/model.rs`)

Add `semantic_weight: f32` to `VerifyConfig`:
- Default: `0.3`
- Valid range: `0.0` to `1.0`
- Values outside this range are rejected by config validation with a clear error message
- `semantic_weight = 0.0` fully disables semantic prediction (identical to current behavior)

### 3. Score Blending (`src/verify/predict.rs`)

In the predictor pipeline, after computing `rule_score` for each test file:
1. Call `compute_semantic_scores` to get semantic scores
2. Blend: `final_score = (1.0 - semantic_weight) * rule_score + semantic_weight * semantic_score`
3. For test files with no semantic score (absent from HashMap): use `semantic_score = 0.0`
4. For test files appearing only in semantic scores (no rule-based coverage): include with `rule_score = 0.0`
5. When `semantic_weight = 0.0`: output is byte-for-byte identical to current predictor
6. When embedding is unavailable: `semantic_score = 0.0` for all tests

### 4. `--explain` Flag (`src/cli.rs`, `src/commands/verify.rs`)

Add `--explain` flag to `VerifyArgs`. When set, print after prediction:

```
Test priority rationale:
  tests/foo.rs    rule: 0.80  semantic: 0.72  final: 0.78
    Semantic basis: 3 of 4 similar past changes caused failures
  tests/bar.rs    rule: 0.60  semantic: 0.00  final: 0.42
    Semantic basis: insufficient history (< 5 samples)
```

Without `--explain`, the rationale table is not printed.

## Test Specifications

| Test | Assertion |
|---|---|
| `compute_semantic_scores` seeded history | High-fail test file gets higher score than low-fail test file |
| `compute_semantic_scores` empty history | Returns empty HashMap |
| `compute_semantic_scores` < 5 history rows | Returns empty HashMap (cold start) |
| Blending with known scores | `final_score` equals weighted formula |
| `semantic_weight = 0.0` | Final scores identical to pre-expansion behavior |
| `semantic_weight = 1.5` | Config validation rejects with error |
| `--explain` output | Includes rule, semantic, and final scores per test |
| Without `--explain` | No rationale table printed |
| Cold start < 50 rows | Warming up message printed |
| Cold start ≥ 50 rows | No warming up message |

## Constraints & Guidelines

- **TDD**: All tests written before implementation.
- **Regression safety**: A dedicated regression test confirms `semantic_weight = 0.0` produces identical output to current predictor.
- **Graceful degradation**: When embedding is unavailable, all `semantic_score` values are `0.0` and prediction is unchanged.
- **Test isolation**: Use `tempfile::tempdir()` for SQLite in tests.

## Hardening Additions (not in original plan)

| Addition | Reason |
|---|---|
| `--no-predict` flag suppresses both rule-based AND semantic prediction | Semantic predictor must respect the existing `--no-predict` flag. If disabled, no predictor code (rule or semantic) should execute. Regression test included. |

## Plan: Track M4-2 — Semantic Predictor & Score Blending

### Phase 1: Semantic Score Computation
- [ ] Task 1.1: Add `compute_semantic_scores(conn: &Connection, query_vec: &[f32], model_name: &str) -> Result<HashMap<String, f32>>` to `src/verify/semantic_predictor.rs`.
- [ ] Task 1.2: Load all `"diff"` entity embeddings from `test_outcome_history` joined to `embeddings`.
- [ ] Task 1.3: For each stored diff embedding, compute `cosine_sim(query_vec, stored_vec)`.
- [ ] Task 1.4: Take the top-30 most similar stored diffs.
- [ ] Task 1.5: For each `test_file` appearing in those 30 diffs' history rows, compute `weighted_score = sum(similarity * (fail_count / total_count))` across all matching history rows.
- [ ] Task 1.6: Normalize scores to [0.0, 1.0] by dividing by the max.
- [ ] Task 1.7: Return as `HashMap<test_file, semantic_score>`.
- [ ] Task 1.8: Write unit test: seed history with known similarity profile; assert high-fail test file gets score > low-fail test file.
- [ ] Task 1.9: Write unit test: empty history → returns empty HashMap.
- [ ] Task 1.10: Write unit test: fewer than 5 history records → returns empty HashMap (cold start threshold).

### Phase 2: Score Blending in `predict.rs`
- [ ] Task 2.1: Add `semantic_weight: f32` to `[verify]` config in `VerifyConfig` (default: 0.3, valid range 0.0–1.0).
- [ ] Task 2.2: Add validation: `semantic_weight` outside [0.0, 1.0] is rejected by config validator with a clear error message.
- [ ] Task 2.3: In the predictor pipeline in `src/verify/predict.rs`, after computing `rule_score` for each test file, call `compute_semantic_scores` and blend: `final_score = (1.0 - semantic_weight) * rule_score + semantic_weight * semantic_score`.
- [ ] Task 2.4: For test files with no semantic score (absent from HashMap), use `semantic_score = 0.0`.
- [ ] Task 2.5: For test files appearing only in semantic scores (no rule-based coverage), include them with `rule_score = 0.0`.
- [ ] Task 2.6: When `semantic_weight = 0.0`, output is identical to current predictor (regression test).
- [ ] Task 2.7: Write unit test: blending with known rule_score and semantic_score produces expected final_score.
- [ ] Task 2.8: Write unit test: `semantic_weight = 0.0` → final scores identical to pre-expansion behavior.
- [ ] Task 2.9: Write unit test: config with `semantic_weight = 1.5` fails validation.

### Phase 3: Cold Start Messaging
- [ ] Task 3.1: Track total history row count in `compute_semantic_scores`.
- [ ] Task 3.2: If history count < 50, print once per `execute_verify` invocation: `Semantic prediction: warming up ({count}/50 history records)`.
- [ ] Task 3.3: Write unit test: 3 history records → warming up message is generated.
- [ ] Task 3.4: Write unit test: 50+ history records → no warming up message.

### Phase 4: `--explain` Flag
- [ ] Task 4.1: Add `--explain` flag to `VerifyArgs` in `src/cli.rs`.
- [ ] Task 4.2: When `--explain` is set, after building the prediction, print a per-test-file rationale table:
  ```
  Test priority rationale:
    tests/foo.rs    rule: 0.80  semantic: 0.72  final: 0.78
      Semantic basis: 3 of 4 similar past changes caused failures
    tests/bar.rs    rule: 0.60  semantic: 0.00  final: 0.42
      Semantic basis: insufficient history (< 5 samples)
  ```
- [ ] Task 4.3: Write unit test: `--explain` output includes rule, semantic, and final scores for each predicted test.
- [ ] Task 4.4: Write unit test: without `--explain`, rationale table is not printed.

### Phase 5: Final Validation
- [ ] Task 5.1: Verify that `--no-predict` flag correctly suppresses both rule-based AND semantic prediction (no predictor code runs at all).
- [ ] Task 5.2: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib verify` — all new tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions, including regression test for `semantic_weight = 0.0`.
- [ ] Task 5.4: Run `changeguard verify --explain` on the changeguard repo; confirm rationale output is printed.

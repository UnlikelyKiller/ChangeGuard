# Track 32: Predictive Verification Completion

## 1. Goal
Complete the predictive verification engine originally scoped in Track 26. This requires implementing real structural prediction (identifying files that import changed files), removing placeholder code, ensuring graceful degradation, fixing deduplication to retain traceability, and thoroughly testing the predictor.

## 2. Requirements

### 2.1. Structural Prediction
- `Predictor::predict()` must perform structural prediction, not just temporal.
- It must identify files that import or depend on the changed files. This can utilize the existing index or tree-sitter capabilities to find reverse-dependencies.
- Combine structural predictions with the existing temporal couplings from the impact packet.

### 2.2. Temporal Analysis Integration
- `commands/verify.rs` must properly integrate temporal analysis before plan construction.
- If temporal data is unavailable (e.g., missing from the packet or history analysis failed), the system must emit deterministic, actionable warnings and gracefully fall back to structural-only prediction.

### 2.3. Traceability and Deduplication
- When deduplicating verification steps, if a predicted command duplicates a direct rule command, the system must retain the `predicted reason` (or merge reasons) so traceability of *why* the prediction was made is not erased.

### 2.4. Code Quality
- Remove all placeholder/thought-process comments in `src/verify/predict.rs` (e.g., "Let's re-read the spec carefully", "For now, let's implement what we CAN...").

### 2.5. Testing
- Add robust unit tests for `Predictor::predict()` covering:
  - Structural prediction behavior.
  - Temporal prediction behavior.
  - Graceful degradation when temporal data is missing.
  - Deduplication retaining traceability.
  - Deterministic ordering of predicted steps.
- Add CLI integration tests proving actual prediction behavior in end-to-end usage.

## 3. Boundaries
- Do not rewrite the entire verification engine; strictly address the gaps in `src/verify/predict.rs`, `src/commands/verify.rs`, and the step deduplication logic.
- Structural prediction implementation should align with the project's existing capabilities (e.g., leveraging `src/index` metrics or standard regex/AST parsing strategies).

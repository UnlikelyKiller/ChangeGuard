# Technical Specification: Track T1 - Predictive CI Gate Analysis & Failure Explanation

## 1. Overview
The goal of Track T1 is to predict Continuous Integration (CI) gate failures locally before a push or PR, leveraging semantic similarity between current file changes and historical change diffs that led to failures. In addition, using the local LLM, the system will provide a natural-language explanation for predicted failures.

## 2. Architecture & Data Model
### 2.1 Database Schema Additions
- **Table: `ci_outcome_history`**
  - `id` (INTEGER PRIMARY KEY)
  - `diff_embedding_id` (INTEGER, FK to `embeddings(id)`)
  - `ci_file_id` (INTEGER, FK to `project_files(id)`)
  - `job_name` (TEXT)
  - `platform` (TEXT)
  - `outcome` (TEXT: 'pass', 'fail', 'skip')
  - `commit_hash` (TEXT)
  - *Note:* Parallels the existing `test_outcome_history` but focuses on CI jobs.

### 2.2 Semantic Prediction Extension
- **Module:** `src/verify/ci_predictor.rs` (or extend `semantic_predictor.rs`)
- `record_ci_outcomes`: Similar to `record_test_outcomes`. It will receive the outcome of CI jobs for a given commit and diff text, store the diff embedding (if not already present), and map it in `ci_outcome_history`.
- `query_similar_ci_outcomes`: Given the current diff text, generate an embedding, find the top K similar historical diffs, and return the aggregated CI job outcomes.

### 2.3 Failure Explanation Generation
- **Module:** `src/verify/explanation.rs`
- **Inputs:**
  - `diff_text`: Summary of current changes.
  - `predicted_failures`: List of CI jobs (or tests) predicted to fail.
  - `ci_gate_details`: Definition of the job from `ci_gates` (e.g., job steps, platform).
  - `historical_context`: The previous similar diffs and their outcomes.
- **Process:** Formulate a structured prompt for the local LLM. Request a concise, deterministic explanation for the likely failure based on the historical precedent and current changes.
- **Output:** Natural-language string explaining the risk.

### 2.4 CLI Integration
- **`impact` Command (`src/commands/impact.rs`):**
  - Incorporate the CI predictor to flag at-risk CI gates in the standard impact summary.
- **`verify` Command (`src/commands/verify.rs`):**
  - Add the `--explain` flag.
  - When invoked, if any CI gate or test is predicted to fail, trigger the explanation generator.
  - Print the generated explanation to the terminal using the application's standard `output` module.

## 3. Implementation Constraints
- **Error Handling:** Strict adherence to `miette::Diagnostic`. No `unwrap()`, `expect()`, or `panic!()`.
- **Local-First:** All embedding and text-generation tasks must utilize the configured local model (`LocalModelConfig`). Fallback gracefully (e.g., return empty explanations) if the model is offline or unresponsive.
- **Windows Resilience:** Ensure all file paths and path-based logic use `PathBuf` and avoid hardcoded Unix path separators.
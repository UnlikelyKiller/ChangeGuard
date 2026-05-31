# Implementation Plan: Track T1 - Predictive CI Gate Analysis & Failure Explanation

## Phase 1: Data Model & Schema Expansion
- [ ] Task 1.1: Add `ci_outcome_history` table creation to database migrations (`src/state/migrations.rs`).
- [ ] Task 1.2: Define `CIJobOutcome` struct to represent the results of CI jobs.
- [ ] Task 1.3: Implement `record_ci_outcomes` (in `src/verify/semantic_predictor.rs` or `ci_predictor.rs`) to store CI run results and link them to diff embeddings.
- [ ] Task 1.4: Implement `query_similar_ci_outcomes` to retrieve historical CI failures based on current diff similarity.

## Phase 2: Failure Explanation Engine
- [ ] Task 2.1: Create the `src/verify/explanation.rs` module.
- [ ] Task 2.2: Implement a prompt builder that constructs a context-aware prompt using current diffs, matched historical diffs, and `ci_gates` configuration.
- [ ] Task 2.3: Integrate with the local model client to fetch the explanation from the local LLM.
- [ ] Task 2.4: Implement graceful degradation; return informative `miette` errors if the LLM generation fails or times out.

## Phase 3: CLI & Command Integration
- [ ] Task 3.1: Update the `impact` pipeline to evaluate and append CI failure predictions to the `ImpactPacket` (or related output structure).
- [ ] Task 3.2: Update `VerifyArgs` in `src/cli.rs` to support the `--explain` boolean flag.
- [ ] Task 3.3: Modify `src/commands/verify.rs` to detect predicted failures and invoke the explanation engine when `--explain` is provided.
- [ ] Task 3.4: Format and display the natural language explanation cleanly in the console output.

## Phase 4: Testing & Hardening
- [ ] Task 4.1: Write unit tests for `record_ci_outcomes` and `query_similar_ci_outcomes` using a mock SQLite connection and HTTP mock for the embeddings server.
- [ ] Task 4.2: Write tests for the prompt builder in `explanation.rs` to ensure determinism and correct context injection.
- [ ] Task 4.3: Perform a codebase audit for this track to verify zero usage of `unwrap()` or `expect()`.
- [ ] Task 4.4: Validate cross-platform path handling, ensuring Windows compatibility.
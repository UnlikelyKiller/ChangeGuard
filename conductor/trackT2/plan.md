# Implementation Plan: Track T2 - Probabilistic Verification Reordering

## Phase 1: Probability Engine Foundation
- [ ] Task 1.1: Create `src/verify/probability.rs`.
- [ ] Task 1.2: Implement native logistic regression (or Bayesian probability) logic for correlating changed files/symbols to failure histories. Ensure deterministic seeding (e.g., seed=42) and capped history limits.
- [ ] Task 1.3: Create unit tests for the math logic using mock datasets (ensuring it accurately sorts targets with known failure patterns).

## Phase 2: SQLite Dataset Extraction
- [ ] Task 2.1: Write the SQLite extraction queries in `src/verify/probability.rs` to fetch `test_outcome_history` (and `ci_outcome_history`) mapped to recently changed file context. Use `i64` for all integer processing per `rusqlite` 0.39.0.
- [ ] Task 2.2: Implement the cold-start safeguard. Count available runs and gracefully abort probability calculation (returning a specific variant or `None`) if `runs < 10`.
- [ ] Task 2.3: Implement the "insufficient variance" safeguard. If all extracted runs passed, safely abort probability calculation with a clear diagnostic.

## Phase 3: Integration into Verification Planner
- [ ] Task 3.1: Modify `src/verify/plan.rs` to accept probability scores and reorder `VerificationCommand`s accordingly (descending order of failure probability).
- [ ] Task 3.2: Update the `changeguard verify` orchestrator in `src/commands/verify.rs` to call the probability engine prior to execution.
- [ ] Task 3.3: Implement explicit console diagnostics when falling back to sequential ordering due to cold-start, lack of variance, or math errors.

## Phase 4: Validation & Hardening
- [ ] Task 4.1: Write integration tests for the cold-start behavior, verifying that the exact `"Probabilistic verification ordering requires at least 10 historical runs..."` diagnostic is printed.
- [ ] Task 4.2: Write tests validating deterministic probability sorting given a fixed fixture database.
- [ ] Task 4.3: Ensure no instances of `unwrap()` or `expect()` are present in the new engine.

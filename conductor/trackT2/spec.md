# Technical Specification: Track T2 - Probabilistic Verification Reordering

## 1. Overview
The goal of Track T2 is to implement Probabilistic Verification (Phase 20 of the Phase 2 implementation plan). It dynamically orders local verification commands using historical failure probabilities based on the symbols changed, rather than blind rule sets. This completes the "Predictable Verification" milestone by ensuring that the tests most likely to fail are run first, minimizing feedback loops locally.

## 2. Architecture & Data Model
### 2.1 Database Integration
- The system will query the existing `test_outcome_history` (and newly added `ci_outcome_history` if applicable) alongside `changed_files` to build a dataset mapping changed symbol features to verification outcome probabilities.
- All integer column queries must use `i64` per `rusqlite` 0.39.0 defaults.

### 2.2 Probabilistic Engine
- **Module:** `src/verify/probability.rs`
- **Algorithm:** Implement logistic regression (or a naive Bayesian estimator) as a native Rust implementation first to avoid the disproportionate dependency weight of `linfa`. If the native version is inadequate for the dataset complexity, use `linfa-logistic` (0.8.1).
- **Determinism:** The model must yield deterministic orderings. If any stochastic initialization is needed, a hardcoded random seed (e.g., `42`) must be used.
- **Cold-Start Strategy:** If there are fewer than 10 completed verification runs in the history database, the probabilistic ordering must be explicitly disabled, emitting the diagnostic: `"Probabilistic verification ordering requires at least 10 historical runs (found: N). Using sequential ordering."`

### 2.3 Integration with Verification Planning
- **Module Update:** `src/verify/plan.rs`
- The `plan.rs` module must be updated to accept an optional probability-ordered plan.
- The `changeguard verify` command will calculate the probabilities, reorder the command plan such that highest-probability failures execute first, and then run them.
- Any model failures (e.g., singular matrix, lack of variance where 100% of historical runs passed) must explicitly fall back to sequential ordering with a clear diagnostic reason.

## 3. Implementation Constraints
- **Local-First & Determinism:** All correlations occur locally using SQLite. The ordering must be perfectly reproducible given the exact same history state. No random variation.
- **Error Handling:** Fallible operations must use `miette::Result`. Failures in the statistical model must NOT crash the CLI; they must result in a fallback to standard sequential verification with a visible `miette::Diagnostic` warning.
- **No Unbounded Memory:** Ensure that large histories are capped (e.g., limit history to the last 1000 runs) during the probability dataset extraction to maintain `verify` execution speed under 2 seconds.

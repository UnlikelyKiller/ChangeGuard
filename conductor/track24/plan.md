# Track 24 Plan: Complexity Indexing

## Phase 1: Spike Evaluation
1. Add `arborist-metrics = "0.1.2"` to `Cargo.toml`.
2. Create a spike test analyzing Phase 1 fixture files.
3. Evaluate compatibility with current `tree-sitter` (0.26.8).
4. Decide: Adopt `arborist-metrics` or implement native fallback.

## Phase 2: Implementation (Native Fallback or Integration)
1. **Trait Definition**:
   - Define `ComplexityScorer` trait in `src/index/metrics.rs`.
2. **Metrics Computation**:
   - If adopting `arborist-metrics`: Implement the trait by wrapping the crate's API.
   - If native: Implement branching/nesting counters using tree-sitter queries.
3. **State Integration**:
   - Update SQLite schema to store complexity scores for symbols.
   - Add migrations for new complexity columns.
4. **Index Integration**:
   - Update the indexer to compute complexity during symbol extraction.

## Phase 3: Verification
1. **Unit Tests**:
   - Compare scores against hand-calculated values for simple/complex functions.
2. **Integration Tests**:
   - Verify complexity persistence in SQLite.
3. **Edge Case Tests**:
   - Files with syntax errors.
   - Unsupported languages.
   - Large files.

## Phase 4: Review & Merge
1. Review by `@rust-triage-specialist`.
2. Push to `main`.

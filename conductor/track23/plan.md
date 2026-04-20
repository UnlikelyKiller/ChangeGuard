# Track 23 Plan: Temporal Intelligence (History Extraction)

## Phase 1: Research & Setup
1. Verify `gix` 0.81.0 API for commit traversal and file listing.
2. Create test fixtures with synthetic git history (using temporary directories).

## Phase 2: Implementation
1. **Module Scaffolding**:
   - Create `src/impact/temporal.rs`.
   - Add module to `src/impact/mod.rs`.
2. **Git Crawl Logic**:
   - Implement `gix` repository opening and OID traversal.
   - Implement file listing for each commit.
   - Filter merge commits and large commits.
3. **Affinity Engine**:
   - Build a map of file co-occurrences.
   - Calculate percentage-based affinity scores.
   - Filter by threshold and sort results.
4. **Integration**:
   - Add `temporal` configuration to `src/config/model.rs`.
   - Update `src/impact/analysis.rs` (or equivalent) to include temporal data in the impact packet.

## Phase 3: Verification
1. **Unit Tests**:
   - Test affinity calculation with mock data.
   - Test sorting and thresholding.
2. **Fixture Tests**:
   - Create a real git repo in a temp dir.
   - Commit files in specific patterns.
   - Run `TemporalCoupling` engine and verify expected pairs.
3. **Edge Case Tests**:
   - Empty repository.
   - Repository with 1 commit.
   - Repository with only merge commits.
   - Shallow clones (simulated).

## Phase 4: Review & Merge
1. Submit for review by `@rust-triage-specialist`.
2. Address feedback.
3. Merge into `main` and push.

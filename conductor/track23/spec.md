# Track 23 Spec: Temporal Intelligence (History Extraction)

## Objective
Identify "Logical Coupling" between files that frequently change together in git history, even when they lack structural imports. This data will be used to enrich the impact packet and identify hotspots.

## Deliverables
- `src/impact/temporal.rs`: Core logic for git history crawling and affinity scoring.
- Integration of `gix` (0.81.0) for repository traversal.
- Unit and fixture tests for affinity calculations.

## Functional Requirements
1. **Commit Crawling**:
   - Use `gix` to traverse commit history starting from `HEAD`.
   - Default depth: 1,000 commits (configurable via `temporal.max_commits`).
   - Traversal mode: First-parent by default.
2. **Exclusion Rules**:
   - Exclude merge commits.
   - Exclude "giant" commits touching >50 files (configurable via `temporal.max_files_per_commit`).
3. **Affinity Scoring**:
   - Calculate the frequency of file B appearing in commits that also contain file A.
   - Threshold for "Coupling": >75% (configurable via `temporal.coupling_threshold`).
4. **Determinism**:
   - Scoring must be deterministic given the same commit history.
   - Results (coupled files) must be sorted alphabetically by path for stable output.
5. **Error Handling**:
   - Handle shallow clones gracefully (return error with `git fetch --unshallow` recommendation).
   - Handle repos with insufficient history (< 10 commits) with a clear diagnostic.
   - Use `miette::Result` for all user-facing errors.

## Internal API
```rust
pub struct TemporalCoupling {
    pub file_a: Utf8PathBuf,
    pub file_b: Utf8PathBuf,
    pub score: f32,
}

pub trait HistoryCrawl {
    fn crawl(&self, depth: usize) -> miette::Result<Vec<TemporalCoupling>>;
}
```

## Dependencies
- `gix = "0.81.0"`
- `miette`
- `serde` (for config/reporting)

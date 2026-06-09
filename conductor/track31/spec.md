# Track 31: Intelligence & Determinism Hardening

## 1. Goal

Address the critical intelligence, determinism, and fidelity gaps identified in Phase 2 for Temporal Coupling, Complexity Indexing, and Hotspot Identification. This track ensures that the core analytical engines are robust, deterministic, accurately follow specified heuristics, and handle edge cases gracefully without panics.

## 2. Core Fixes Required

### 2.1 Temporal Intelligence Hardening (Track 23 Gaps)
*   **First-Parent Traversal**: `src/impact/temporal.rs` must use first-parent traversal by default, not `Sorting::BreadthFirst`.
*   **All-Parents Flag**: Introduce an opt-in `--all-parents` flag (or config option) to enable full graph traversal.
*   **Threshold Semantics**: Align the coupling threshold logic. If the spec requires `>75%`, ensure the implementation uses strictly greater than (or adjust spec documentation if `>=` is explicitly desired, but standardizing to the strict `>` is preferred per audit).
*   **Error Handling**: Gix unparseable commit errors should not be fatal `GitError::MetadataError`. They must be treated as partial failures, appropriately logged/annotated, and allow traversal to continue.
*   **Git Fixture Testing**: Implement a synthetic git repository fixture test (not just an in-memory mock) to validate temporal coupling logic.

### 2.2 Complexity Indexing Hardening (Track 24 Gaps)
*   **Spike Documentation**: Document the `arborist-metrics` spike decision (even retroactively) as an ADR in `docs/architecture/` or a dedicated doc.
*   **Degradation Modeling**: 
    *   Add an `ast_incomplete: bool` field to `FileComplexity` (set to `true` if `tree.root_node().has_error()`).
    *   Support `Complexity::NotApplicable` (or similar) for unsupported languages.
    *   Implement a large-file cap (e.g., files > 10,000 lines) and set a `complexity_capped: bool` flag.
*   **TypeScript Support**: Extend tree-sitter queries to match `method_definition` and other common TS nodes.
*   **Safety**: Remove the production `.unwrap()` in `src/commands/impact.rs` (`Utf8Path::from_path(relative_path).unwrap()`).
*   **Testing**: Implement robust golden-value tests with hand-calculated expected scores, replacing weak `> 1` assertions. Add specific TS tests.

### 2.3 Hotspot Identification Hardening (Track 25 Gaps)
*   **CLI Enhancements**: Add missing `changeguard hotspots` CLI options: JSON output (`--json`), directory/language filtering (`--dir`, `--lang`), and logical-neighbor display.
*   **Scoring Formula**: Correct the risk density formula to match `docs/Plan-Phase2.md`: Normalized multiplication using `value / max(all_values)`. Avoid unconfigurable 50/50 sums.
*   **Determinism & Safety**: 
    *   Sort deterministically by score, with a fallback tie-breaker on the file path.
    *   Handle `NaN` values safely during sorting (do not panic with `.unwrap()`).
*   **Error Handling**: Do not silently drop SQLite row errors with `.filter_map(|res| res.ok())`. Surface them appropriately.
*   **Code Deduplication**: Remove duplicated hotspot logic in `src/commands/hotspots.rs`. It must call the shared engine in `src/impact/hotspots.rs`.
*   **Testing**: Add dedicated hotspot scoring math tests and deterministic ranking tests.

## 3. Interfaces & Contracts

### 3.1 Complexity Data Model
```rust
pub enum Complexity {
    Score(FileComplexity),
    NotApplicable, // For unsupported languages
}

pub struct FileComplexity {
    pub cyclomatic: f64,
    pub cognitive: f64,
    pub ast_incomplete: bool,  // true if tree-sitter encountered parse errors
    pub complexity_capped: bool, // true if file exceeded line limit
}
```

### 3.2 Hotspot Sorting Tie-breaker
```rust
// Example robust sorting
a.score.partial_cmp(&b.score)
    .unwrap_or(std::cmp::Ordering::Equal)
    .then_with(|| a.path.cmp(&b.path))
```

## 4. Acceptance Criteria
1. `cargo test` passes, including new git fixture and complexity golden tests.
2. Hotspots correctly normalize scores against maximum values and sort deterministically without panics.
3. Complexity gracefully degrades on syntax errors, large files, and unsupported languages.
4. Temporal traversal respects first-parent by default and does not abort on single malformed commits.
5. No production `.unwrap()` or `.expect()` calls remain in these modules.
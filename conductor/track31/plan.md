# Plan: Track 31 - Intelligence & Determinism Hardening

### Phase 1: Temporal Intelligence Hardening
- [ ] Task 1.1: Modify `src/impact/temporal.rs` traversal to default to first-parent instead of `Sorting::BreadthFirst`.
- [ ] Task 1.2: Add an opt-in `--all-parents` flag to temporal analysis configuration and CLI logic.
- [ ] Task 1.3: Update temporal coupling threshold logic to strictly follow spec semantics (e.g., `> threshold`).
- [ ] Task 1.4: Refactor `gix` error handling in temporal traversal to treat unparseable commits as partial failures (skip and log) rather than returning fatal `GitError::MetadataError`.
- [ ] Task 1.5: Create a real synthetic git repository fixture for `tests/temporal_coupling.rs` and validate first-parent traversal and coupling output.

### Phase 2: Complexity Indexing Hardening
- [ ] Task 2.1: Write a short ADR or documentation entry for the `arborist-metrics` spike decision.
- [ ] Task 2.2: Update `FileComplexity` struct to include `ast_incomplete` and `complexity_capped` booleans. Add `Complexity::NotApplicable` enum variant.
- [ ] Task 2.3: Modify `NativeComplexityScorer` to detect `tree.root_node().has_error()` (setting `ast_incomplete`), cap parsing at 10,000 lines (setting `complexity_capped`), and return `NotApplicable` for unknown languages.
- [ ] Task 2.4: Update tree-sitter queries to properly capture TypeScript nodes (e.g., `method_definition`).
- [ ] Task 2.5: Remove the `Utf8Path::from_path(relative_path).unwrap()` in `src/commands/impact.rs` and properly handle the path conversion error.
- [ ] Task 2.6: Refactor `tests/complexity_scoring.rs` to use hand-calculated golden values for assertions instead of simple `> 1` checks. Add TypeScript specific tests.

### Phase 3: Hotspot Identification Hardening
- [ ] Task 3.1: Deduplicate hotspot calculation logic by removing it from `src/commands/hotspots.rs` and fully deferring to `src/impact/hotspots.rs`.
- [ ] Task 3.2: Update hotspot risk density formula to use normalized multiplication (`(freq / max_freq) * (complex / max_complex)`) instead of hardcoded 50/50 sums.
- [ ] Task 3.3: Implement deterministic sorting in hotspots with a path-based tiebreaker and safe `NaN` handling (removing `.unwrap()`).
- [ ] Task 3.4: Fix silent SQLite error drops in the hotspot query (`.filter_map(|res| res.ok())`); propagate or log row errors properly.
- [ ] Task 3.5: Add missing CLI options to `changeguard hotspots`: `--json`, `--dir`, `--lang`.
- [ ] Task 3.6: Write comprehensive deterministic ranking and math tests for hotspot calculation.
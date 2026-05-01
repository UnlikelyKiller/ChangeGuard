## Plan: Track E2-4 - Critical Path Analysis

### Phase 1: Database Schema
- [ ] Task 1.1: Add `symbol_centrality` table creation to Migration M16 in `src/state/migrations.rs` (shared with E2-1, E2-2, E2-3). Columns: `id`, `symbol_id` (INTEGER NOT NULL REFERENCES project_symbols(id)), `file_id` (INTEGER NOT NULL REFERENCES project_files(id)), `entrypoints_reachable`, `betweenness`, `last_computed_at`. Include indices on `symbol_id`, `file_id`, and `entrypoints_reachable`.
- [ ] Task 1.2: Add `symbol_centrality` to the `test_all_tables_exist` test in `src/state/migrations.rs`.
- [ ] Task 1.3: Write a new test `test_insert_and_query_symbol_centrality` verifying insertion, retrieval, and index usage.

### Phase 2: Centrality Computation Module
- [ ] Task 2.1: Create `src/index/centrality.rs` with a `CentralityComputer` struct that takes a database connection and loads `structural_edges` and entry-point-labeled `project_symbols`.
- [ ] Task 2.2: Implement adjacency-list builder: load all rows from `structural_edges` (using `caller_symbol_id` and `callee_symbol_id` foreign keys), construct a `HashMap<i64, Vec<i64>>` mapping each caller symbol ID to its callee symbol IDs (forward edges).
- [ ] Task 2.3: Implement BFS reachability: for each entry point (from `project_symbols` where `entrypoint_kind IN ('ENTRYPOINT', 'HANDLER')`), perform BFS through the adjacency list with a visited set. Increment `entrypoints_reachable` for each symbol encountered.
- [ ] Task 2.4: Implement BFS depth cap: limit traversal to 20 hops from each entry point. Symbols beyond 20 hops are not counted as reachable.
- [ ] Task 2.5: Implement BFS cycle safety: use a `HashSet` visited set per entry point to prevent revisiting symbols.
- [ ] Task 2.6: Implement reachability cap: if a single entry point reaches more than 50,000 symbols, cap the BFS and log a warning.
- [ ] Task 2.7: Implement optional betweenness centrality: count how many shortest paths from entry points pass through each symbol. Store as `betweenness` (approximate, using sampled paths if graph is large).
- [ ] Task 2.8: Implement graceful skip: if `structural_edges` is empty or no entry points exist, log info and return without computation.
- [ ] Task 2.9: Implement storage: clear existing `symbol_centrality` rows and batch-insert new results in transactions of 500 rows.
- [ ] Task 2.10: Write unit tests for `CentralityComputer`: empty edges (skip), single entry point with chain, multiple entry points sharing a callee, cycle handling, depth-cap enforcement, reachability-cap enforcement.

### Phase 3: Index Command Integration
- [ ] Task 3.1: Add centrality computation step to `changeguard index --analyze-graph` (not the standard `index` run). Centrality is only computed when the `--analyze-graph` flag is provided, after `structural_edges` (E2-1) and entry point labeling (E1-4) are complete. Call `CentralityComputer::compute()` with the database connection.
- [ ] Task 3.2: Add `--analyze-graph` flag to `changeguard index` that triggers centrality computation. A standard `changeguard index` run does NOT compute centrality.
- [ ] Task 3.3: Verify that centrality is recomputed from scratch on each `index --analyze-graph` run (not incrementally updated). Clear the `symbol_centrality` table before recomputation.
- [ ] Task 3.4: Log an info message when centrality computation is skipped due to missing `structural_edges` or entry points.

### Phase 4: Hotspots Integration
- [ ] Task 4.1: In `src/impact/hotspots.rs`, add an optional `centrality` column to the hotspot output format. When `symbol_centrality` has data, include `entrypoints_reachable` for each file/symbol.
- [ ] Task 4.2: Implement `--centrality` flag on `changeguard hotspots` that explicitly requests centrality data in the output.
- [ ] Task 4.3: Implement JSON output: when `--json` is used and centrality data exists, include `centrality` and `entrypoints_reachable` fields in the serialized output.
- [ ] Task 4.4: Implement fallback: when `structural_edges` is empty or `symbol_centrality` is empty, display "Centrality: N/A" in text output and omit centrality fields in JSON output.
- [ ] Task 4.5: Write integration tests for hotspots with centrality: verify centrality column appears, verify JSON includes centrality fields, verify "N/A" when no data.

### Phase 5: Impact Integration
- [ ] Task 5.1: In `src/impact/analysis.rs`, add a `centrality_risk` function that queries `symbol_centrality` (joining on `symbol_id`) for changed symbols and applies up to 15 points within the Historical Hotspot category (max 30 points) when `entrypoints_reachable > 5`.
- [ ] Task 5.2: Add risk reason: `"High centrality: {symbol_name} reachable from {N} entry points"`.
- [ ] Task 5.3: Integrate `centrality_risk` into the `analyze_risk` pipeline after structural-coupling risk (E2-1) and data-model risk (E2-3). If `symbol_centrality` table is empty, skip.
- [ ] Task 5.4: Make the centrality threshold (default: 5 entry points) configurable via the risk scoring configuration.
- [ ] Task 5.5: Write integration tests: changing a symbol with `entrypoints_reachable > 5` produces the centrality risk reason and up to 15 points within the Historical Hotspot category.

### Phase 6: End-to-End Testing
- [ ] Task 6.1: Create a fixture project with 5 route handlers (entry points) that all call a shared `process_request()` function. Run `changeguard index --analyze-graph`, verify `symbol_centrality` has `entrypoints_reachable = 5` for `process_request`.
- [ ] Task 6.2: Run `changeguard hotspots --centrality` on the fixture (after `changeguard index --analyze-graph`). Verify the centrality column appears with correct reachability counts.
- [ ] Task 6.3: Run `changeguard impact` on a change to `process_request`. Verify risk reason includes `"High centrality: process_request reachable from 5 entry points"` and up to 15 points within the Historical Hotspot category (max 30 points).
- [ ] Task 6.4: Create a fixture with a cyclic call graph (A -> B -> A). Run centrality computation. Verify no infinite loop.
- [ ] Task 6.5: Run `changeguard hotspots` without `--centrality` on the fixture. Verify output matches baseline (no centrality column).
- [ ] Task 6.6: Run `changeguard impact` on a repo with no `structural_edges` data. Verify no centrality-related risk reasons and no regressions.
- [ ] Task 6.7: Performance test: centrality computation on a call graph with 10,000 edges and 50 entry points completes in under 5 seconds.
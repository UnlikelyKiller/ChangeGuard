## Plan: Track E0-2 Hotspot Complexity Fallback

### Phase 1: Extract and Test the Complexity Query
- [ ] Task 1.1: Extract the inline SQL query in `calculate_hotspots` into a new function `query_file_complexities` in `src/impact/hotspots.rs`. It takes a `&rusqlite::Connection`, queries the `symbols` table, and returns `HashMap<String, i32>`.
- [ ] Task 1.2: Write failing test `test_hotspots_uses_symbols_when_available`: Create an in-memory SQLite DB, insert rows into `symbols` with known complexity values for specific file paths, call `calculate_hotspots` (or `query_file_complexities`), and assert the returned complexity matches.
- [ ] Task 1.3: Write failing test `test_hotspots_graceful_degradation_without_project_symbols`: Create an in-memory DB with `symbols` data but no `project_symbols` table, call the function, verify it returns `symbols` data without crashing.
- [ ] Task 1.4: Run tests. Confirm `test_hotspots_uses_symbols_when_available` and existing hotspot tests pass (no regression from refactor).

### Phase 2: Implement the Fallback Logic
- [ ] Task 2.1: Extend `query_file_complexities` to accept an optional `fallback_table: Option<&str>`. When provided, attempt to query that table after the primary query.
- [ ] Task 2.2: Implement per-file gap-fill merge: start with the primary (`symbols`) results, then iterate the fallback (`project_symbols`) results. Only insert entries for files that are not already present in the primary map. Primary takes precedence for any file present in both tables. This is NOT an all-or-nothing fallback -- if symbols has data for 5 files and project_symbols has data for 10 different files, the merged result contains all 15 files.
- [ ] Task 2.3: Wrap the fallback query in error handling. If the query fails with a "no such table" error, log `tracing::debug!("project_symbols table not available, skipping fallback")` and return primary results only. If it fails for any other reason, propagate the error.
- [ ] Task 2.4: Update `calculate_hotspots` to call `query_file_complexities` with `Some("project_symbols")` as the fallback table.

### Phase 3: Fallback Tests
- [ ] Task 3.1: Write and pass test `test_hotspots_falls_back_to_project_symbols`: Create an in-memory DB with `project_symbols` data but no entries in `symbols` (or an empty `symbols` result for the target files). Verify that the fallback data populates the complexity map.
- [ ] Task 3.2: Write and pass test `test_hotspots_prefers_symbols_over_project_symbols`: Create both tables with different complexity values for the same file path. Verify that the `symbols` value is used.
- [ ] Task 3.3: Write and pass test `test_hotspots_empty_project_symbols_no_crash`: Create the `project_symbols` table but leave it empty. Verify the function returns only `symbols` data (no crash, no wrong results).

### Phase 4: Integration and Regression
- [ ] Task 4.1: Run `cargo test` and confirm all existing tests pass, including the full hotspot test suite.
- [ ] Task 4.2: Run `cargo clippy` and resolve any new warnings.
- [ ] Task 4.3: Manual smoke test: on a repo with no prior `impact` run (no `symbols` data), verify `changeguard hotspots` completes without error and returns meaningful results (or warns that no index data is available).
- [ ] Task 4.4: Manual smoke test: on a repo with prior `impact` data, verify `changeguard hotspots` returns the same results as before (primary data unaffected).
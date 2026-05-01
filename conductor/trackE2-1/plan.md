## Plan: Track E2-1 - Call Graph and Structural Coupling

### Phase 1: Database Schema
- [ ] Task 1.1: Add Migration M16 to `src/state/migrations.rs` creating the `structural_edges` table with columns `id`, `caller_symbol_id` (INTEGER NOT NULL REFERENCES project_symbols(id)), `caller_file_id` (INTEGER NOT NULL REFERENCES project_files(id)), `callee_symbol_id` (INTEGER REFERENCES project_symbols(id)), `callee_file_id` (INTEGER REFERENCES project_files(id)), `unresolved_callee` (TEXT, nullable), `call_kind`, `resolution_status` (TEXT NOT NULL DEFAULT 'RESOLVED'), `confidence` (REAL NOT NULL DEFAULT 1.0), `evidence` (TEXT, nullable), and indices on `caller_symbol_id+caller_file_id` and `callee_symbol_id+callee_file_id`.
- [ ] Task 1.2: Add `structural_edges` to the `test_all_tables_exist` test in `src/state/migrations.rs`.
- [ ] Task 1.3: Write a new test `test_insert_and_query_structural_edges` in `src/state/migrations.rs` that inserts DIRECT, METHOD_CALL, and DYNAMIC edges with `resolution_status`, `confidence`, and `evidence` values and verifies retrieval.

### Phase 2: Call Extraction - Rust
- [ ] Task 2.1: Add a `extract_calls` function to `src/index/languages/rust.rs` that walks `call_expression` and `method_call_expression` nodes in the tree-sitter AST.
- [ ] Task 2.2: Implement callee resolution for Rust: match callee name against `project_symbols` entries, producing `caller_symbol_id`, `callee_symbol_id`, `caller_file_id`, `callee_file_id` foreign keys. Distinguish `DIRECT` (free function calls), `METHOD_CALL` (receiver.method calls), and `TRAIT_DISPATCH` (trait-object calls). Mark unresolved as `DYNAMIC` with `resolution_status = 'UNRESOLVED'` and `unresolved_callee` populated. Set `confidence` based on call kind.
- [ ] Task 2.3: Write unit tests in `src/index/languages/rust.rs` for call extraction: direct function call, method call, trait dispatch, and unresolved dynamic call.

### Phase 3: Call Extraction - TypeScript
- [ ] Task 3.1: Add a `extract_calls` function to `src/index/languages/typescript.rs` that walks `call_expression` and `new_expression` nodes.
- [ ] Task 3.2: Implement callee resolution for TypeScript: match against `project_symbols`. Mark `new` expressions as `METHOD_CALL`. Mark callback invocations as `DYNAMIC`.
- [ ] Task 3.3: Write unit tests in `src/index/languages/typescript.rs` for call extraction: named function call, method call, new expression, and dynamic callback.

### Phase 4: Call Extraction - Python
- [ ] Task 4.1: Add a `extract_calls` function to `src/index/languages/python.rs` that walks `call` nodes in function bodies.
- [ ] Task 4.2: Implement callee resolution for Python: match against `project_symbols`. Mark `obj.method()` as `METHOD_CALL`. Mark `getattr()` and other dynamic dispatch as `DYNAMIC`.
- [ ] Task 4.3: Write unit tests in `src/index/languages/python.rs` for call extraction: function call, method call, dynamic dispatch, and cross-module call.

### Phase 5: Call Graph Builder Module
- [ ] Task 5.1: Create `src/index/call_graph.rs` with a `CallGraphBuilder` struct that queries `project_symbols`, dispatches to language-specific call extractors, and collects edges.
- [ ] Task 5.2: Implement resolution logic: for each extracted call, attempt to match callee name + file path against `project_symbols`, producing `callee_symbol_id` and `callee_file_id` foreign keys. Unmatched calls become `DYNAMIC` edges with `unresolved_callee` set and `resolution_status = 'UNRESOLVED'`. Ambiguous matches set `resolution_status = 'AMBIGUOUS'`. Cross-language references become `EXTERNAL`.
- [ ] Task 5.3: Implement batched streaming inserts: insert edges into `structural_edges` in batches of 500 with transaction commits.
- [ ] Task 5.4: Implement edge cap: if a file produces more than 50,000 edges, log a warning and cap at 50,000, prioritizing edges involving public symbols.
- [ ] Task 5.5: Implement graceful skip: if `project_symbols` is empty, log info and return without error.
- [ ] Task 5.6: Write unit tests for `CallGraphBuilder`: empty project_symbols (skip), single-file resolution, multi-file resolution, edge-cap enforcement, batch-insert verification.

### Phase 6: Index Command Integration
- [ ] Task 6.1: Add call graph extraction step to `changeguard index` after `project_symbols` is populated. Call `CallGraphBuilder::build()` with the database connection.
- [ ] Task 6.2: Add `--skip-call-graph` flag to `changeguard index` for users who want symbol indexing without call graph construction (performance opt-out).
- [ ] Task 6.3: Verify incremental indexing clears and rebuilds `structural_edges` only for re-indexed files, not the entire table.

### Phase 7: Impact Integration
- [ ] Task 7.1: In `src/impact/analysis.rs`, add a function `structural_coupling_risk` that queries `structural_edges` for callers of changed symbols and produces risk reasons of the form `"Structurally coupled: {caller} calls {callee}"`.
- [ ] Task 7.2: Integrate `structural_coupling_risk` into the `analyze_risk` pipeline. If `structural_edges` table is empty, skip the query.
- [ ] Task 7.3: Write integration tests: changing a callee symbol produces a risk reason mentioning its callers.

### Phase 8: Verify Integration
- [ ] Task 8.1: In `src/verify/predict.rs`, add a `structural_prediction` function that queries `structural_edges` for callers of changed symbols and returns them as predicted verification targets.
- [ ] Task 8.2: Integrate `structural_prediction` into the `predict` pipeline after import-based prediction and before temporal-coupling prediction.
- [ ] Task 8.3: Write integration tests: changing a callee symbol predicts its callers as verification targets.

### Phase 9: End-to-End Testing
- [ ] Task 9.1: Create a fixture Rust project with `main()` -> `helper()` -> `internal()` call chain. Run `changeguard index`, verify `structural_edges` contains both edges.
- [ ] Task 9.2: Run `changeguard impact` on a change to `internal()`. Verify risk reasons include `"Structurally coupled: helper calls internal"`.
- [ ] Task 9.3: Run `changeguard verify` on a change to `internal()`. Verify the prediction includes `helper` as a verification target.
- [ ] Task 9.4: Run `changeguard impact` and `changeguard verify` on a repo with no `structural_edges` data. Verify output matches baseline (no regression).
- [ ] Task 9.5: Performance test: run call graph extraction on a 500-file fixture repo. Verify completion under 10 seconds.
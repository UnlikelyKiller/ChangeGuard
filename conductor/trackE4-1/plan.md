## Plan: Track E4-1 Test-to-Symbol Mapping

### Phase 1: Database Schema
- [ ] Task 1.1: Add migration M18 to `src/state/migrations.rs` creating the `test_mapping` table with columns `id`, `test_file`, `test_symbol`, `tested_file`, `tested_symbol`, `confidence`, `mapping_kind`, `last_indexed_at` and indices on `test_file`, `tested_file`, and `tested_symbol`.
- [ ] Task 1.2: Write a test verifying the `test_mapping` table is created and supports insert/query operations, including querying by `tested_file` and `tested_symbol`.
- [ ] Task 1.3: Add `TestCoverage` and `CoveringTest` structs to `src/impact/packet.rs`.
- [ ] Task 1.4: Add `test_coverage: Vec<TestCoverage>` field to `ImpactPacket` with `#[serde(default)]`.
- [ ] Task 1.5: Write tests verifying `TestCoverage` and `CoveringTest` serialization/deserialization.

### Phase 2: Test Detection
- [ ] Task 2.1: Create `src/index/test_mapping.rs` module with a `TestMapping` struct holding `test_file`, `test_symbol`, `tested_file`, `tested_symbol`, `confidence`, `mapping_kind`.
- [ ] Task 2.2: Implement Rust test detection: find `#[test]` and `#[tokio::test]` annotated functions, extract function names and module paths.
- [ ] Task 2.3: Implement TypeScript test detection: find `describe()`, `it()`, and `test()` blocks, extract descriptions and imported modules.
- [ ] Task 2.4: Implement Python test detection: find `def test_*()` and `def *_test()` functions, plus `class Test*` classes, extract function/class names.
- [ ] Task 2.5: Write unit tests for each language's test detector using fixture code snippets with known test patterns.

### Phase 3: Symbol Resolution and Mapping
- [ ] Task 3.1: Implement import analysis for test files: extract `use`/`import`/`from ... import` statements and resolve them to `project_symbols` entries.
- [ ] Task 3.2: Implement naming convention analysis: match test names like `test_<symbol>` or `<Symbol>Test` to symbol names in `project_symbols`.
- [ ] Task 3.3: Implement mapping assignment: create IMPORT-based mappings (confidence 1.0) when direct imports exist, NAMING_CONVENTION-based mappings (confidence 0.6) when only naming matches exist.
- [ ] Task 3.4: Handle integration tests that import multiple modules: create multiple mapping rows with reduced confidence (0.7).
- [ ] Task 3.5: Write unit tests for symbol resolution: verify that a Rust test importing `foo::bar` maps to symbol `bar` in file `foo.rs`.
- [ ] Task 3.6: Write unit tests for naming convention: verify that `test_foo_bar` maps to symbol `foo_bar` with `mapping_kind = 'NAMING_CONVENTION'`.

### Phase 4: Index Integration
- [ ] Task 4.1: Add `extract_test_mappings` function to `src/index/test_mapping.rs` that dispatches to language-specific test detectors and resolves mappings.
- [ ] Task 4.2: Wire test mapping extraction into `src/commands/index.rs`: after symbol extraction, call `extract_test_mappings` for files in Test directories (from `project_topology`).
- [ ] Task 4.3: Insert mapping results into `test_mapping` table. On re-index, delete existing rows for a file before inserting new ones.
- [ ] Task 4.4: Write integration test: run `changeguard index` on a fixture repo with Rust, TypeScript, and Python test files, and verify `test_mapping` rows are populated correctly.

### Phase 5: Verify Integration
- [ ] Task 5.1: Modify `src/verify/predict.rs` to query `test_mapping` for changed symbols before running temporal coupling and structural import predictions.
- [ ] Task 5.2: Add test-mapping-based predictions as Priority 1 in the verification plan with reason "Test mapping: X tests Y".
- [ ] Task 5.3: Order predictions: IMPORT-based mappings (confidence 1.0) before NAMING_CONVENTION-based mappings (confidence 0.6) before temporal and structural predictions.
- [ ] Task 5.4: Write test: changing symbol `bar()` in a fixture repo predicts running `test_bar` as Priority 1 via test mapping.
- [ ] Task 5.5: Write test: IMPORT-based mapping predictions appear before NAMING_CONVENTION-based predictions in the verification plan.

### Phase 6: Impact Integration
- [ ] Task 6.1: Modify `src/impact/analysis.rs` to query `test_mapping` for changed symbols and build `TestCoverage` entries.
- [ ] Task 6.2: Add `TestCoverage` entries to `ImpactPacket.test_coverage` for each changed symbol that has test mappings.
- [ ] Task 6.3: When no test coverage exists for a changed symbol, add an advisory: "No test coverage found for X".
- [ ] Task 6.4: Write test: changing a symbol with test mappings produces a `test_coverage` entry in the impact JSON report listing the covering tests.
- [ ] Task 6.5: Write test: changing a symbol without test mappings produces an advisory about missing coverage.

### Phase 7: Final Validation
- [ ] Task 7.1: Run full test suite (`cargo test`) and verify no regressions in existing `impact`, `hotspots`, `verify`, or `ledger` tests.
- [ ] Task 7.2: Run `changeguard index` on a fixture repo with test files and verify `test_mapping` rows are created with correct confidence values and mapping kinds.
- [ ] Task 7.3: Run `changeguard verify` on a fixture repo and verify Priority 1 test-mapping predictions appear before temporal and structural predictions.
- [ ] Task 7.4: Run `changeguard impact` on a fixture repo and verify `test_coverage` appears in JSON output when test mappings exist.
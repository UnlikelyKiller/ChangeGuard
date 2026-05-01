## Plan: Track E3-1 Logging and Event Pattern Detection

### Phase 1: Database Schema
- [ ] Task 1.1: Add migration M17 to `src/state/migrations.rs` creating the `observability_patterns` table with columns `id`, `file_path`, `line_start`, `pattern_kind`, `level`, `framework`, `in_test`, `last_indexed_at` and indices on `file_path` and `pattern_kind`.
- [ ] Task 1.2: Write a test verifying the `observability_patterns` table is created and supports insert/query operations.
- [ ] Task 1.3: Add `CoverageDelta` struct to `src/impact/packet.rs` with fields `file_path`, `pattern_kind`, `previous_count`, `current_count`, `message` and derive `Serialize`, `Deserialize`, `Clone`.
- [ ] Task 1.4: Add `logging_coverage_delta: Vec<CoverageDelta>` field to `ImpactPacket` with `#[serde(default)]`.

### Phase 2: Logging Pattern Extraction
- [ ] Task 2.1: Create `src/index/observability.rs` module with a `LoggingPattern` struct holding `file_path`, `line_start`, `level`, `framework`, `in_test`.
- [ ] Task 2.2: Implement Rust logging extraction using tree-sitter queries to find macro invocations matching `log::*!`, `tracing::*!`, `println!`, and `eprintln!` patterns.
- [ ] Task 2.3: Implement TypeScript logging extraction using tree-sitter queries to find `console.*`, `logger.*`, and `winston.*` call expressions.
- [ ] Task 2.4: Implement Python logging extraction using tree-sitter queries to find `logging.*`, `logger.*`, and `print()` call expressions.
- [ ] Task 2.5: Implement `in_test` detection for each language: Rust `#[test]` functions, TypeScript `describe`/`it`/`test` blocks, Python `test_*` functions.
- [ ] Task 2.6: Write unit tests for each language extractor using fixture code snippets with known logging patterns.
- [ ] Task 2.7: Write unit tests verifying `in_test` detection correctly identifies logging inside test functions in all three languages.

### Phase 3: Index Integration
- [ ] Task 3.1: Add `extract_logging_patterns` function to `src/index/observability.rs` that dispatches to the language-specific extractors.
- [ ] Task 3.2: Wire logging extraction into `src/commands/index.rs`: after symbol extraction, call `extract_logging_patterns` and insert results into `observability_patterns` with `pattern_kind = 'LOG'`.
- [ ] Task 3.3: Implement upsert logic: on re-index, delete existing `LOG` rows for a file before inserting new ones.
- [ ] Task 3.4: Write integration test: run `changeguard index` on a fixture repo and verify `observability_patterns` rows are populated correctly.

### Phase 4: Impact Integration
- [ ] Task 4.1: Add logging coverage delta computation to `src/impact/analysis.rs`: count logging statements in the current file version vs. the stored version in `observability_patterns`.
- [ ] Task 4.2: When coverage decreases, create a `CoverageDelta` entry with message "Logging coverage reduced in X: N statements removed" and add it to the `ImpactPacket`.
- [ ] Task 4.3: Exclude test-file logging (`in_test = true`) from the coverage count comparison.
- [ ] Task 4.4: Write test: removing a `tracing::info!()` call from a Rust fixture produces a `logging_coverage_delta` entry in the impact JSON output.
- [ ] Task 4.5: Write regression test: `changeguard impact` on a repo with no logging patterns produces an empty `logging_coverage_delta` without errors.

### Phase 5: Final Validation
- [ ] Task 5.1: Run full test suite (`cargo test`) and verify no regressions in existing `impact`, `hotspots`, `verify`, or `ledger` tests.
- [ ] Task 5.2: Run `changeguard index` on the ChangeGuard repo itself and verify `observability_patterns` rows are created.
- [ ] Task 5.3: Run `changeguard impact` and verify `logging_coverage_delta` appears in JSON output when logging statements change.
## Plan: Track E3-2 Error Handling Pattern Detection

### Phase 1: ImpactPacket Extension
- [ ] Task 1.1: Add `error_handling_delta: Vec<CoverageDelta>` field to `ImpactPacket` in `src/impact/packet.rs` with `#[serde(default)]`.
- [ ] Task 1.2: Write test verifying `error_handling_delta` serializes to JSON correctly and defaults to empty when absent.

### Phase 2: Error Handling Pattern Extraction
- [ ] Task 2.1: Implement Rust error handling extraction in `src/index/observability.rs`: detect `match` on `Result`/`Option`, `.unwrap()`, `.expect()`, `?` operator, `anyhow!` macro, and `#[derive(Error)]` attribute.
- [ ] Task 2.2: Implement TypeScript error handling extraction: detect `try/catch/finally` blocks, `.catch()` calls, `Promise.reject` expressions, and `throw` statements.
- [ ] Task 2.3: Implement Python error handling extraction: detect `try/except/finally` blocks, `raise` statements, and `assert` statements.
- [ ] Task 2.4: Implement `in_test` detection for error handling patterns: Rust `#[test]` functions, TypeScript `describe`/`it`/`test` blocks, Python `test_*` functions.
- [ ] Task 2.5: Write unit tests for each language extractor using fixture code snippets with known error handling patterns (match on Result, unwrap, try/catch, try/except).
- [ ] Task 2.6: Write unit tests verifying `.unwrap()` inside test functions is marked `in_test = true` and excluded from risk calculations.

### Phase 3: Index Integration
- [ ] Task 3.1: Add `extract_error_handling_patterns` function to `src/index/observability.rs` that dispatches to language-specific extractors and returns patterns with `pattern_kind = 'ERROR_HANDLE'`.
- [ ] Task 3.2: Wire error handling extraction into `src/commands/index.rs`: after logging extraction, call `extract_error_handling_patterns` and insert results into `observability_patterns`.
- [ ] Task 3.3: Implement upsert logic: on re-index, delete existing `ERROR_HANDLE` rows for a file before inserting new ones.
- [ ] Task 3.4: Write integration test: run `changeguard index` on a fixture repo with Rust, TypeScript, and Python files containing error handling constructs, and verify `observability_patterns` rows are populated correctly.

### Phase 4: Impact Integration - Coverage Delta
- [ ] Task 4.1: Add error handling coverage delta computation to `src/impact/analysis.rs`: count error handling patterns in the current file version vs. the stored version in `observability_patterns`.
- [ ] Task 4.2: When error handling coverage decreases, create a `CoverageDelta` entry with message "Error handling reduced in X: N patterns removed" and add it to the `ImpactPacket.error_handling_delta`.
- [ ] Task 4.3: Implement specific detection: when a `match` on `Result` is replaced with `.unwrap()`, emit "Error handling reduced: unwrap replaces match in X".
- [ ] Task 4.4: Exclude test-file error handling patterns (`in_test = true`) from coverage count comparisons.
- [ ] Task 4.5: Write test: removing a `match` on `Result` and replacing it with `.unwrap()` in a Rust fixture produces a warning about error handling reduction.
- [ ] Task 4.6: Write test: removing a `try/catch` block in a TypeScript fixture produces a coverage delta warning.

### Phase 5: Impact Integration - Risk Weight
- [ ] Task 5.1: Extend `analyze_risk()` in `src/impact/analysis.rs` to detect error handling changes in files classified under `Infrastructure` directories (from `project_topology`).
- [ ] Task 5.2: Apply +25 risk weight to files where error handling patterns changed AND the file is in an Infrastructure directory. Add "Error handling change in infrastructure: X" to `risk_reasons`.
- [ ] Task 5.3: Write test: changing error handling in an Infrastructure directory file produces a +25 risk weight and appropriate risk reason.
- [ ] Task 5.4: Write test: changing error handling in a Source directory file does NOT produce the +25 risk weight.

### Phase 6: Final Validation
- [ ] Task 6.1: Run full test suite (`cargo test`) and verify no regressions in existing `impact`, `hotspots`, `verify`, or `ledger` tests.
- [ ] Task 6.2: Run `changeguard index` on a fixture repo with Rust error handling patterns and verify `observability_patterns` rows with `pattern_kind = 'ERROR_HANDLE'` are created.
- [ ] Task 6.3: Run `changeguard impact` on a fixture repo and verify `error_handling_delta` appears in JSON output when error handling changes.
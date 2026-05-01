## Plan: Track E3-3 Telemetry and Trace Wiring

### Phase 1: Telemetry Pattern Extraction
- [ ] Task 1.1: Implement Rust telemetry extraction in `src/index/observability.rs`: detect `#[instrument]` and `#[tracing::instrument]` attributes, `opentelemetry::` references, `prometheus::` macro calls, and `metrics::` crate calls.
- [ ] Task 1.2: Implement TypeScript telemetry extraction: detect `@Trace()` decorators, `opentelemetry` imports/usage, `prom-client`/`prometheus` usage, and `metrics` module calls.
- [ ] Task 1.3: Implement Python telemetry extraction: detect `@tracer.start_as_current_span` decorators, `opentelemetry` imports/usage, `prometheus_client` usage, and `metrics` module calls.
- [ ] Task 1.4: Implement custom telemetry wrapper detection: detect `telemetry.*` and `monitoring.*` patterns in all three languages, storing with `framework = 'custom'`.
- [ ] Task 1.5: Write unit tests for each language extractor using fixture code snippets with known telemetry patterns.
- [ ] Task 1.6: Write unit tests verifying that `#[instrument]` is stored as `pattern_kind = 'TRACE'` (not double-counted as LOG).

### Phase 2: Index Integration
- [ ] Task 2.1: Add `extract_telemetry_patterns` function to `src/index/observability.rs` that dispatches to language-specific extractors and returns patterns with `pattern_kind = 'TRACE'`.
- [ ] Task 2.2: Wire telemetry extraction into `src/commands/index.rs`: after error handling extraction, call `extract_telemetry_patterns` and insert results into `observability_patterns`.
- [ ] Task 2.3: Implement upsert logic: on re-index, delete existing `TRACE` rows for a file before inserting new ones.
- [ ] Task 2.4: Write integration test: run `changeguard index` on a fixture repo with telemetry patterns and verify `observability_patterns` rows with `pattern_kind = 'TRACE'` are created.

### Phase 3: Impact Integration - Telemetry Coverage Delta
- [ ] Task 3.1: Add telemetry coverage delta computation to `src/impact/analysis.rs`: count `TRACE` patterns in the current file version vs. the stored version in `observability_patterns`.
- [ ] Task 3.2: When telemetry coverage decreases, create a `CoverageDelta` entry with `pattern_kind = 'TRACE'` and message "Telemetry coverage reduced in X: N instrumentation points removed".
- [ ] Task 3.3: Write test: removing a `#[instrument]` attribute from a Rust fixture produces a telemetry coverage delta warning.
- [ ] Task 3.4: Write test: removing an `opentelemetry` import from a TypeScript fixture produces a telemetry coverage delta warning.

### Phase 4: Impact Integration - Telemetry Coverage Flag
- [ ] Task 4.1: Add `--telemetry-coverage` flag to the `impact` command in `src/commands/impact.rs`.
- [ ] Task 4.2: When `--telemetry-coverage` is enabled, query `api_routes` and `observability_patterns` to find files with API routes or handler functions that have zero `TRACE` patterns.
- [ ] Task 4.3: Surface the list of files missing telemetry as advisory warnings in the impact output.
- [ ] Task 4.4: Write test: `changeguard impact --telemetry-coverage` on a fixture repo with API routes but no telemetry produces an advisory warning about missing telemetry.

### Phase 5: LSP Integration
- [ ] Task 5.1: Extend the LSP hover provider in `src/lsp/` to query `observability_patterns` by `file_path` and count patterns grouped by `pattern_kind`.
- [ ] Task 5.2: Display observability pattern counts in the hover response, formatted as "N log statements, N error handlers, N trace".
- [ ] Task 5.3: Ensure the LSP hover integration is behind the existing feature flag and degrades gracefully when `observability_patterns` has no data for a file.
- [ ] Task 5.4: Write test: LSP hover for a file with observability patterns includes the pattern count summary.

### Phase 6: Final Validation
- [ ] Task 6.1: Run full test suite (`cargo test`) and verify no regressions in existing `impact`, `hotspots`, `verify`, or `ledger` tests.
- [ ] Task 6.2: Run `changeguard index` on a fixture repo with telemetry patterns and verify `observability_patterns` rows with all three `pattern_kind` values (LOG, ERROR_HANDLE, TRACE) are populated.
- [ ] Task 6.3: Run `changeguard impact --telemetry-coverage` and verify it surfaces files missing telemetry.
- [ ] Task 6.4: Verify LSP daemon starts and serves hover information with observability pattern counts when the feature flag is enabled.
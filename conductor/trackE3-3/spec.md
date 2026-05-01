# Specification: Track E3-3 Telemetry and Trace Wiring

## Overview

Implement the third track of Phase E3 (Observability Wiring) from `docs/expansion-plan.md`. This track detects OpenTelemetry, Prometheus, and custom metrics instrumentation in source code, extends the `observability_patterns` table with `pattern_kind = 'TRACE'`, and integrates with both the `impact` command (warning when telemetry is removed) and the LSP daemon (showing observability pattern counts on hover).

## Motivation

Telemetry instrumentation (spans, metrics, traces) provides production visibility. When developers remove telemetry annotations like `#[instrument]` or `opentelemetry` calls, the system's observability degrades. ChangeGuard currently has no awareness of telemetry patterns. This track makes telemetry visible, flags regressions, and surfaces observability information in the LSP.

## Components

### 1. Telemetry Pattern Extraction (`src/index/observability.rs`)

Extend the existing observability module with telemetry/trace pattern detection.

**Rust patterns to detect:**
- `#[instrument]` and `#[tracing::instrument]` attributes: detect as `TRACE` with `framework = 'tracing'`
- `#[otel::instrument]` or similar OpenTelemetry attributes: detect as `TRACE` with `framework = 'opentelemetry'`
- `opentelemetry::` module references (function calls, use statements): detect as `TRACE` with `framework = 'opentelemetry'`
- `prometheus::` macro calls (`histogram_observed!`, `gauge!`, `counter!`, `inc!`, `observe!`): detect as `TRACE` with `framework = 'prometheus'`
- `metrics::` crate calls (`counter!`, `gauge!`, `histogram!`): detect as `TRACE` with `framework = 'metrics'`

**TypeScript patterns to detect:**
- `@Trace()` decorator: detect as `TRACE` with `framework = 'opentelemetry'`
- `opentelemetry` import and usage: detect as `TRACE` with `framework = 'opentelemetry'`
- `prom-client` or `prometheus` usage (`new Counter`, `new Histogram`, `new Gauge`): detect as `TRACE` with `framework = 'prom-client'`
- `metrics` module calls: detect as `TRACE` with `framework = 'metrics'`

**Python patterns to detect:**
- `@tracer.start_as_current_span` or `@tracer.start_span` decorators: detect as `TRACE` with `framework = 'opentelemetry'`
- `opentelemetry` import and usage: detect as `TRACE` with `framework = 'opentelemetry'`
- `prometheus_client` usage (`Counter`, `Gauge`, `Histogram`, `Summary`): detect as `TRACE` with `framework = 'prometheus_client'`
- `metrics` module calls: detect as `TRACE` with `framework = 'metrics'`

**Custom telemetry wrappers:**
- Common patterns like `telemetry.*`, `monitoring.*` method calls: detect as `TRACE` with `framework = 'custom'` and `confidence = 0.7`. Custom telemetry wrappers are heuristic detections and must not be presented as confirmed. Always label them clearly.

**Extraction output per pattern:**
- `file_id`: integer foreign key referencing `project_files(id)` for the source file containing the telemetry instrumentation
- `line_start`: line number of the telemetry construct
- `pattern_kind`: `'TRACE'`
- `level`: one of `info` (standard instrumentation), `debug` (verbose spans), `error` (error-level metrics)
- `framework`: one of `tracing`, `opentelemetry`, `prometheus`, `prom-client`, `prometheus_client`, `metrics`, or `custom`
- `confidence`: REAL NOT NULL DEFAULT 1.0 — 1.0 for known framework calls, 0.7 for custom telemetry wrappers (`framework = 'custom'`)
- `evidence`: TEXT — human-readable evidence string, e.g. `"attribute: #[instrument]"`, `"call: opentelemetry::tracer()"`, `"heuristic: telemetry.* pattern match"`

### 2. Database Schema (`src/state/migrations.rs`)

The `observability_patterns` table is already created by migration M17 (Track E3-1). This track uses the same table with `pattern_kind = 'TRACE'`. No new migration is needed. Note that the table uses `file_id INTEGER NOT NULL REFERENCES project_files(id)` rather than `file_path TEXT`, and includes `confidence REAL NOT NULL DEFAULT 1.0` and `evidence TEXT` columns per expansion plan constraints.

### 3. Index Integration (`src/commands/index.rs`)

Wire telemetry extraction into the `changeguard index` command:

- After logging and error handling patterns are extracted for a file, run telemetry pattern extraction on the same AST.
- Insert results into `observability_patterns` with `pattern_kind = 'TRACE'`.
- On re-index, delete existing `TRACE` rows for the file before inserting new ones.

### 4. Impact Integration (`src/impact/analysis.rs`)

Add telemetry coverage delta detection:

- Count telemetry patterns in the current file version versus the stored version in `observability_patterns`.
- When telemetry is removed (count decreases), add a `CoverageDelta` entry to `ImpactPacket.logging_coverage_delta` (reusing the existing field) with message: "Telemetry coverage reduced in X: N instrumentation points removed".
- The `pattern_kind` field in the `CoverageDelta` will be `'TRACE'` to distinguish telemetry deltas from logging deltas.

**Optional `--telemetry-coverage` flag:**

- Add `--telemetry-coverage` flag to the `impact` command.
- When enabled, surface files that *should* have telemetry but don't (files with API routes or handler functions that have zero `TRACE` patterns). This is a heuristic check, not a strict requirement.

### 5. LSP Integration (`src/lsp/`)

Extend the LSP daemon's hover provider to show observability pattern counts:

- When a user hovers over a file, display the count of observability patterns in that file, grouped by kind: "3 log statements, 2 error handlers, 1 trace".
- This uses the `observability_patterns` table queried by `file_id` (joined through `project_files`).
- The hover information is additive and does not replace existing hover content.
- The LSP feature is behind the existing daemon feature flag (the same flag that enables the LSP daemon). No new feature flag is needed.

## Constraints & Guidelines

- **Graceful degradation**: If a language has no telemetry extraction queries, skip it silently. Never crash on missing data.
- **Library telemetry exemption**: Do not flag changes to telemetry instrumentation in library/dependency code (files in `vendor/`, `third_party/`, or `node_modules/` directories).
- **No false confidence**: Label custom telemetry wrapper detection with `framework = 'custom'` and `confidence = 0.7`. Never present heuristic detection as certain.
- **Risk weight**: Telemetry coverage reduction contributes to the **Observability Reduction** category (max 25 points) per expansion plan Section 4.2.
- **TDD Requirement**: Write or update tests for extraction logic, coverage delta computation, LSP hover integration, and the `--telemetry-coverage` flag.
- **No performance regression**: Telemetry extraction must not add more than 5% overhead to the `index` command beyond what logging and error handling extraction already add.
- **Backward-compatible**: The `observability_patterns` table schema is unchanged. Only new rows with `pattern_kind = 'TRACE'` are added. The `--telemetry-coverage` flag is additive.

## Edge Cases

- **`#[instrument]` macro attributes**: Detect as `TRACE` with `framework = 'tracing'`. These are both logging and tracing; store only one entry with `pattern_kind = 'TRACE'` to avoid double-counting.
- **Telemetry in libraries**: Do not flag changes to library telemetry code. Skip files in `vendor/`, `third_party/`, `node_modules/` directories.
- **Missing telemetry in new code**: The `--telemetry-coverage` flag surfaces files with zero `TRACE` patterns that contain API routes or handler functions. This is advisory only.
- **Custom telemetry wrappers**: Detect `telemetry.*`, `monitoring.*` patterns as `TRACE` with `framework = 'custom'`. These may produce false positives; label them clearly.
- **Multiple telemetry frameworks in one file**: Store each detected pattern as a separate row. A file may have both `tracing` and `prometheus` instrumentation.

## Acceptance Criteria

- `changeguard index` populates `observability_patterns` with `pattern_kind = 'TRACE'` entries for Rust, TypeScript, and Python source files.
- `changeguard impact` warns when telemetry coverage decreases in a changed file.
- `changeguard impact --telemetry-coverage` surfaces files that should have telemetry but don't (when API routes or handlers are present without TRACE patterns).
- LSP hover shows observability pattern counts per file (when the LSP daemon feature flag is enabled).
- Existing commands remain unaffected by the new flag and patterns.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M17 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates observability_patterns for fixture repos
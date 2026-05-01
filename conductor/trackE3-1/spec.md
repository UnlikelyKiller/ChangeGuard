# Specification: Track E3-1 Logging and Event Pattern Detection

## Overview

Implement the first track of Phase E3 (Observability Wiring) from `docs/expansion-plan.md`. This track detects logging statements in source code, catalogs them in a new `observability_patterns` database table, and integrates with the `impact` command to warn when logging coverage decreases in changed files.

## Motivation

Logging is a primary observability mechanism. When a developer removes or reduces logging statements in a file, production visibility degrades silently. ChangeGuard currently has no awareness of logging patterns. This track makes logging a first-class tracked artifact and ensures changes that reduce logging coverage are flagged in impact analysis.

## Components

### 1. Logging Pattern Extraction (`src/index/observability.rs`)

New module that extracts logging statements from source code using tree-sitter queries and heuristic pattern matching.

**Rust patterns to detect:**
- `log::info!(...)`, `log::warn!(...)`, `log::error!(...)`, `log::debug!(...)`, `log::trace!(...)`
- `tracing::info!(...)`, `tracing::warn!(...)`, `tracing::error!(...)`, `tracing::debug!(...)`, `tracing::trace!(...)`
- `println!(...)`, `eprintln!(...)`
- Any macro call matching `*::info!`, `*::warn!`, `*::error!`, `*::debug!`, `*::trace!`

**TypeScript patterns to detect:**
- `console.log(...)`, `console.warn(...)`, `console.error(...)`, `console.info(...)`, `console.debug(...)`
- `logger.info(...)`, `logger.warn(...)`, `logger.error(...)`, `logger.debug(...)`
- `winston.log(...)`, `winston.info(...)`, `winston.warn(...)`, `winston.error(...)`

**Python patterns to detect:**
- `logging.info(...)`, `logging.warning(...)`, `logging.error(...)`, `logging.debug(...)`, `logging.critical(...)`
- `logger.info(...)`, `logger.warning(...)`, `logger.error(...)`, `logger.debug(...)`, `logger.critical(...)`
- `print(...)` (only at module level or in non-test functions)

**Extraction output per pattern:**
- `file_id`: integer foreign key referencing `project_files(id)` for the source file containing the logging statement
- `line_start`: line number of the logging call
- `level`: one of `debug`, `info`, `warn`, `error`, `trace`
- `framework`: one of `log`, `tracing`, `console`, `logging`, `winston`, `println`, `print`, or `custom`
- `confidence`: REAL NOT NULL DEFAULT 1.0 — confidence score for the detection (1.0 for exact macro/call matches, lower for heuristic matches)
- `evidence`: TEXT — human-readable evidence string, e.g. `"macro: tracing::info!"`, `"call: console.log"`
- `in_test`: boolean indicating whether the logging statement is inside a test function

### 2. Database Schema (`src/state/migrations.rs`)

Add migration M17 to create the `observability_patterns` table:

```sql
CREATE TABLE IF NOT EXISTS observability_patterns (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES project_files(id),
    line_start INTEGER,
    pattern_kind TEXT NOT NULL DEFAULT 'LOG',
    level TEXT,
    framework TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    evidence TEXT,
    in_test INTEGER DEFAULT 0,
    last_indexed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_obs_patterns_file ON observability_patterns(file_id);
CREATE INDEX IF NOT EXISTS idx_obs_patterns_kind ON observability_patterns(pattern_kind);
```

The `pattern_kind` column uses the values `LOG`, `ERROR_HANDLE`, and `TRACE` to support the subsequent E3-2 and E3-3 tracks. This track populates rows with `pattern_kind = 'LOG'`.

### 3. Index Integration (`src/index/mod.rs`, `src/commands/index.rs`)

Wire logging extraction into the `changeguard index` command:

- After symbols and runtime_usage are extracted for a file, run logging pattern extraction on the same AST.
- Insert results into `observability_patterns` with `pattern_kind = 'LOG'`.
- On re-index, delete existing `LOG` rows for the file before inserting new ones (upsert pattern).

### 4. Impact Integration (`src/impact/analysis.rs`)

Add logging coverage delta detection to the `impact` analysis pipeline:

- After a file is analyzed, count the logging statements in the current version (from the changed file's AST) versus the version at git HEAD (using `gix` diff). This is a comparison against git HEAD, not against a stored snapshot, keeping it simple and always current.
- If the count decreases, add a `CoverageDelta` entry to the `ImpactPacket.logging_coverage_delta` field with message: "Logging coverage reduced in X: N statements removed".
- Logging statements in test files (where `in_test = true`) are excluded from the coverage count.

**Risk weight:** Logging coverage reduction contributes to the **Observability Reduction** category (max 25 points) in the category-capped scoring model (expansion plan Section 4.2). Each logging coverage delta adds up to 25 points to this category, capped at the category maximum.

### 5. ImpactPacket Extension (`src/impact/packet.rs`)

Add the `logging_coverage_delta` field to `ImpactPacket`:

```rust
#[serde(default)]
pub logging_coverage_delta: Vec<CoverageDelta>,
```

Where `CoverageDelta` is:

```rust
pub struct CoverageDelta {
    pub file_path: String,    // display only; join via file_id for identity
    pub pattern_kind: String,  // "LOG", "ERROR_HANDLE", "TRACE"
    pub previous_count: usize,
    pub current_count: usize,
    pub message: String,
}
```

All new fields must have `#[serde(default)]` to maintain backward compatibility with existing JSON consumers.

## Constraints & Guidelines

- **Graceful degradation**: If a language has no logging extraction queries, skip it silently. Missing data is a visible warning in logs, never a crash.
- **No false confidence**: Do not count commented-out logging statements. Only count active code.
- **Test file exemption**: Logging statements inside `#[test]` functions, `describe()`/`it()` blocks, or `test_*` functions are marked `in_test = true` and excluded from coverage metrics.
- **TDD Requirement**: Write or update tests for extraction logic, migration schema, coverage delta computation, and ImpactPacket serialization.
- **No performance regression**: Logging extraction must not add more than 10% overhead to the `index` command for typical repos.
- **Backward-compatible schema**: The `observability_patterns` table is new and additive. No existing tables are modified.

## Edge Cases

- **Commented-out logging**: Do not count. Tree-sitter should naturally skip these since they are comment nodes, not expression nodes.
- **Macro-generated logging** (e.g., `#[instrument]`): Detect the attribute and count it as `pattern_kind = 'TRACE'` (deferred to E3-3 for full handling).
- **Logging in test files**: Mark with `in_test = true`. Exclude from coverage delta calculations.
- **Very large files with many logging statements**: Cap extraction at 1000 patterns per file to avoid excessive memory usage.
- **Dynamic log levels** (e.g., `log!(level, ...)`): Store `level` as `null` and `framework` as the resolved crate name if possible, `custom` otherwise.

## Acceptance Criteria

- `changeguard index` populates `observability_patterns` with `pattern_kind = 'LOG'` entries for Rust, TypeScript, and Python source files.
- `changeguard impact` warns when logging coverage decreases in a changed file by emitting a `CoverageDelta` in the JSON report.
- Test files' logging patterns are stored with `in_test = true` and excluded from coverage metrics.
- Existing `impact`, `hotspots`, `verify`, and `ledger` commands remain unaffected by this change.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M17 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates observability_patterns for fixture repos
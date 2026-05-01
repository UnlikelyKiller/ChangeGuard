# Specification: Track E4-1 Test-to-Symbol Mapping

## Overview

Implement the first track of Phase E4 (Safety Context) from `docs/expansion-plan.md`. This track maps test functions to the symbols they test, adds a `test_mapping` table (Migration M18), and integrates with the `verify` command (Priority 1 prediction) and `impact` command (show test coverage).

## Motivation

ChangeGuard's verification prediction (`verify`) currently uses structural imports and temporal coupling to suggest which tests to run. It does not know which test function tests which symbol. This means predictions are indirect and often imprecise. By mapping tests directly to the symbols they exercise, `verify` can produce Priority 1 predictions: "if you change symbol X, run test Y." Similarly, `impact` can report which tests cover a changed symbol, giving developers immediate confidence about verification.

## Components

### 1. Test Detection and Mapping (`src/index/test_mapping.rs`)

New module that detects test functions and maps them to the symbols they test.

**Rust test detection:**
- Detect `#[test]` and `#[tokio::test]` attribute annotations on functions.
- Extract the function name and the module path.
- Map to tested symbols by analyzing `use` statements and function calls within the test body.
- Confidence: `IMPORT` (high) when the test directly imports the tested module; `NAMING_CONVENTION` (medium) when the test name follows `test_<symbol>` convention; `COVERAGE_DATA` (high) when coverage data maps the test to the symbol.

**TypeScript test detection:**
- Detect `describe()`, `it()`, and `test()` blocks with their descriptions.
- Map to tested symbols by analyzing `import` statements that reference the tested module.
- Confidence: `IMPORT` (high) when the test directly imports the tested module; `NAMING_CONVENTION` (medium) when the test description matches a module or function name; `COVERAGE_DATA` (high) when coverage data maps the test to the symbol.

**Python test detection:**
- Detect `def test_*()` and `def *_test()` functions, plus `class Test*` with `def test_*()` methods.
- Map to tested symbols by analyzing `import` statements that reference the tested module.
- Confidence: `IMPORT` (high) when the test directly imports the tested module; `NAMING_CONVENTION` (medium) when the function name follows `test_<symbol>` convention; `COVERAGE_DATA` (high) when coverage data maps the test to the symbol.

**Mapping algorithm:**
1. Identify test functions via language-specific patterns.
2. For each test function, extract all import/use statements in the same file.
3. Resolve imported symbols to `project_symbols` entries.
4. Create a mapping row for each (test_function, tested_symbol) pair.
5. Assign `mapping_kind = 'IMPORT'` when a direct import relationship exists, `mapping_kind = 'NAMING_CONVENTION'` when only a naming pattern matches, `mapping_kind = 'COVERAGE_DATA'` when coverage data maps test to symbol.
6. Assign `confidence = 1.0` for IMPORT mappings, `confidence = 0.5` for NAMING_CONVENTION mappings (matching expansion plan Section 4/E4-1).

### 2. Database Schema (`src/state/migrations.rs`)

Add migration M18 to create the `test_mapping` table:

```sql
CREATE TABLE IF NOT EXISTS test_mapping (
    id INTEGER PRIMARY KEY,
    test_symbol_id INTEGER NOT NULL REFERENCES project_symbols(id),
    test_file_id INTEGER NOT NULL REFERENCES project_files(id),
    tested_symbol_id INTEGER REFERENCES project_symbols(id),
    tested_file_id INTEGER REFERENCES project_files(id),
    confidence REAL NOT NULL DEFAULT 1.0,
    mapping_kind TEXT NOT NULL DEFAULT 'IMPORT',
    evidence TEXT,
    last_indexed_at TEXT NOT NULL,
    UNIQUE(test_symbol_id, tested_symbol_id)
);
CREATE INDEX IF NOT EXISTS idx_test_mapping_tested ON test_mapping(tested_symbol_id);
CREATE INDEX IF NOT EXISTS idx_test_mapping_test ON test_mapping(test_symbol_id);
```

### 3. Index Integration (`src/commands/index.rs`)

Wire test mapping into the `changeguard index` command:

- After symbol extraction, run test detection on all files in Test directories (from `project_topology`).
- For each detected test function, resolve imports to `project_symbols` entries and create mapping rows.
- On re-index, delete existing `test_mapping` rows for a file before inserting new ones.

### 4. Verify Integration (`src/verify/predict.rs`)

Add test-mapping-based prediction as Priority 1 (before temporal and structural prediction):

- When a file is changed, query `test_mapping` for all tests that map to symbols in the changed file.
- Add these tests to the verification plan with reason: "Test mapping: X tests Y".
- Priority order: IMPORT-based mappings (confidence = 1.0) come before NAMING_CONVENTION-based mappings (confidence = 0.5).
- Test-mapping predictions appear before temporal coupling and structural import predictions in the verification plan.

### 5. Impact Integration (`src/impact/analysis.rs`)

Add test coverage information to the impact report:

- When a changed symbol has test mappings, add a `TestCoverage` entry to `ImpactPacket.test_coverage` with the list of test functions that cover the symbol.
- Display in JSON report as: "Tests covering this change: X" where X is the test function name(s).
- When no test coverage exists for a changed symbol, add an advisory: "No test coverage found for X".

**Cross-track dependencies:**
- E1-1 (`project_symbols`): Test mapping relies on `project_symbols` for resolving test and tested symbol IDs. Without `project_symbols`, test mapping must degrade gracefully (fall back to text-based name matching with reduced confidence).
- E1-3 (`project_topology`): Test detection uses `project_topology` to identify Test directories. Without topology data, fall back to heuristic path matching (`tests/`, `test/`, `spec/`, `__tests__/`).

### 6. ImpactPacket Extension (`src/impact/packet.rs`)

Add the `test_coverage` field to `ImpactPacket`:

```rust
#[serde(default)]
pub test_coverage: Vec<TestCoverage>,
```

Where `TestCoverage` is:

```rust
pub struct TestCoverage {
    pub changed_symbol: String,
    pub changed_file: String,
    pub covering_tests: Vec<CoveringTest>,
}

pub struct CoveringTest {
    pub test_file: String,
    pub test_symbol: String,
    pub confidence: f64,
    pub mapping_kind: String,
}
```

All new fields must have `#[serde(default)]` to maintain backward compatibility.

## Constraints & Guidelines

- **Deterministic over speculative**: If a test-to-symbol mapping cannot be confirmed with high confidence, label it `NAMING_CONVENTION` with reduced confidence rather than presenting it as fact. Never present speculation as confirmed.
- **Graceful degradation**: If no test files exist in the repo, skip test mapping entirely. No warnings about missing test coverage for repos that don't have tests.
- **Integration tests**: Tests that test multiple modules create multiple mappings with reduced confidence. Do not try to pick one "best" mapping.
- **TDD Requirement**: Write or update tests for test detection, mapping resolution, verify prediction integration, and impact report integration.
- **No performance regression**: Test mapping extraction must not add more than 15% overhead to the `index` command.
- **Backward-compatible schema**: The `test_mapping` table is new and additive. No existing tables are modified.

## Edge Cases

- **Integration tests testing multiple modules**: Create multiple mapping rows, one per imported symbol, each with reduced confidence (0.7 for IMPORT).
- **Tests with no clear import relationship**: Mark as `mapping_kind = 'NAMING_CONVENTION'` with confidence 0.6. Example: a test named `test_foo_bar` in a file that does not import `foo_bar`.
- **No test files in the repo**: Skip test mapping. No warnings, no errors. The `test_mapping` table remains empty.
- **Test files in a separate directory** (Rust `tests/` at repo root): Use module path resolution to map the test file to the crate/module it imports.
- **Test helper functions** (not test cases): Functions in test files that are not marked as tests should not be mapped. Only detect `#[test]`, `describe`/`it`/`test` blocks, and `def test_*` functions.

## Acceptance Criteria

- `changeguard index` populates `test_mapping` for Rust, TypeScript, and Python test files.
- `changeguard verify` includes test-mapping-based predictions as Priority 1 in the verification plan.
- `changeguard impact` shows test coverage information in the JSON report when available.
- IMPORT-based mappings have higher confidence than NAMING_CONVENTION-based mappings.
- Repos without test files continue to function normally with empty test mapping.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M18 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E4 tables for fixture repos
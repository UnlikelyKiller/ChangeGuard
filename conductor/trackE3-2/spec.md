# Specification: Track E3-2 Error Handling Pattern Detection

## Overview

Implement the second track of Phase E3 (Observability Wiring) from `docs/expansion-plan.md`. This track detects error handling patterns in source code (try/catch, Result matching, unwrap, etc.), extends the `observability_patterns` table with `pattern_kind = 'ERROR_HANDLE'`, and integrates with the `impact` command to warn when error handling is reduced and to apply elevated risk weights for error handling changes in Infrastructure directories.

## Motivation

Error handling is a safety net for production systems. When a developer replaces careful `Result` matching with `.unwrap()`, or removes a `try/catch` block, the system becomes more fragile. ChangeGuard currently has no visibility into error handling patterns, so these regressions go undetected. This track makes error handling observable and flags dangerous changes.

## Components

### 1. Error Handling Pattern Extraction (`src/index/observability.rs`)

Extend the existing observability module with error handling pattern detection.

**Rust patterns to detect:**
- `match` on `Result` or `Option` variants: `match x { Ok(v) => ..., Err(e) => ... }` and `match x { Some(v) => ..., None => ... }`
- `.unwrap()` calls: detect as `ERROR_HANDLE` with `level = 'error'` and `framework = 'unwrap'`
- `.expect("message")` calls: detect as `ERROR_HANDLE` with `level = 'warn'` and `framework = 'expect'`
- `?` operator usage: detect as `ERROR_HANDLE` with `level = 'info'` and `framework = 'try_operator'`
- `anyhow!` macro calls: detect as `ERROR_HANDLE` with `framework = 'anyhow'`
- `thiserror` derive attributes: detect `#[derive(Error)]` as `ERROR_HANDLE` with `framework = 'thiserror'`

**TypeScript patterns to detect:**
- `try/catch/finally` blocks: detect as `ERROR_HANDLE` with `framework = 'try_catch'`
- `.catch()` method calls: detect as `ERROR_HANDLE` with `framework = 'promise_catch'`
- `Promise.reject` expressions: detect as `ERROR_HANDLE` with `framework = 'promise_reject'`
- `throw` statements: detect as `ERROR_HANDLE` with `framework = 'throw'`

**Python patterns to detect:**
- `try/except/finally` blocks: detect as `ERROR_HANDLE` with `framework = 'try_except'`
- `raise` statements: detect as `ERROR_HANDLE` with `framework = 'raise'`
- `assert` statements: detect as `ERROR_HANDLE` with `framework = 'assert'`

**Extraction output per pattern:**
- `file_id`: integer foreign key referencing `project_files(id)` for the source file containing the error handling construct
- `line_start`: line number of the construct
- `pattern_kind`: `'ERROR_HANDLE'`
- `level`: one of `info` (careful handling), `warn` (partial handling like `expect`), `error` (risky handling like `unwrap`)
- `framework`: language-specific identifier for the error handling pattern
- `confidence`: REAL NOT NULL DEFAULT 1.0 — confidence score for the detection
- `evidence`: TEXT — human-readable evidence string for how the pattern was detected, e.g. `"syntactic: match expression"`, `"syntactic: unwrap call"`, `"syntactic: try/catch block"`. Mark all tree-sitter-detected patterns explicitly as syntactic evidence since we cannot always determine whether a `match` is on `Result` or `Option` without type information.
- `in_test`: boolean indicating whether the pattern is inside a test function

### 2. Database Schema (`src/state/migrations.rs`)

The `observability_patterns` table is already created by migration M17 (Track E3-1). This track uses the same table with `pattern_kind = 'ERROR_HANDLE'`. No new migration is needed. Note that the table uses `file_id INTEGER NOT NULL REFERENCES project_files(id)` rather than `file_path TEXT`, and includes `confidence REAL NOT NULL DEFAULT 1.0` and `evidence TEXT` columns per expansion plan constraints.

### 3. Index Integration (`src/commands/index.rs`)

Wire error handling extraction into the `changeguard index` command:

- After logging patterns are extracted for a file, run error handling pattern extraction on the same AST.
- Insert results into `observability_patterns` with `pattern_kind = 'ERROR_HANDLE'`.
- On re-index, delete existing `ERROR_HANDLE` rows for the file before inserting new ones.

### 4. Impact Integration (`src/impact/analysis.rs`)

Add two error-handling-specific impact behaviors:

**4a. Coverage delta detection:**
- Count error handling patterns in the current file version versus the stored version in `observability_patterns`.
- When coverage decreases, add a `CoverageDelta` entry to `ImpactPacket.error_handling_delta` with message: "Error handling reduced in X: N patterns removed".
- When a `match` on `Result` is replaced with `.unwrap()`, emit a specific warning: "Error handling reduced: unwrap replaces match in X".

**4b. Risk weight for Infrastructure directories:**
- Changes to error handling patterns in files classified under `Infrastructure` directories (from `project_topology` table, Track E1-3) contribute to the **Runtime/Config** risk category (max 25 points per expansion plan Section 4.2).
- This is in addition to the topology elevation that Infrastructure files already receive.
- Add "Error handling change in infrastructure: X" to `risk_reasons` when applicable.

**Cross-track dependency:** This track depends on E1-3's `project_topology` table for classifying directories as `Infrastructure`. If `project_topology` data is not available, Infrastructure directory detection must degrade gracefully (fall back to heuristic path matching: `.github/workflows/`, `infra/`, `deploy/`, `terraform/`).

### 5. ImpactPacket Extension (`src/impact/packet.rs`)

Add the `error_handling_delta` field to `ImpactPacket`:

```rust
#[serde(default)]
pub error_handling_delta: Vec<CoverageDelta>,
```

Reuses the `CoverageDelta` struct defined in Track E3-1.

## Constraints & Guidelines

- **Graceful degradation**: If a language has no error handling extraction queries, skip it silently. Never crash on missing data.
- **Test file exemption**: `.unwrap()` and `.expect()` calls inside test functions are not flagged as risk reductions. Test code commonly uses unwrap as a deliberate pattern.
- **No false confidence**: Only flag reductions in error handling, not additions. Adding more error handling is always acceptable.
- **TDD Requirement**: Write or update tests for extraction logic, coverage delta computation, and risk weight application.
- **No performance regression**: Error handling extraction must not add more than 10% overhead to the `index` command beyond what logging extraction already adds.
- **Backward-compatible**: The `observability_patterns` table schema is unchanged. Only new rows with `pattern_kind = 'ERROR_HANDLE'` are added.

## Edge Cases

- **`.unwrap()` in test files**: Not flagged. Mark with `in_test = true` and exclude from risk calculations.
- **`.expect("message")` vs `.unwrap()`**: `expect` is slightly better (documents intent) but still flagged as a reduction when it replaces a `match` on `Result`. Use `level = 'warn'` instead of `level = 'error'`.
- **Error handling in generated code**: Skip files with common generation markers (e.g., `// Generated by`, `// @generated`) in the first N lines.
- **Nested try/catch**: Count each `try` block as one error handling pattern, regardless of nesting depth.
- **Dynamic error handling** (e.g., `catch (e) { if (...) }`): Count the `catch` block as one pattern. Do not analyze the conditional logic inside.

## Acceptance Criteria

- `changeguard index` populates `observability_patterns` with `pattern_kind = 'ERROR_HANDLE'` entries for Rust, TypeScript, and Python source files.
- `changeguard impact` warns when error handling coverage decreases in a changed file.
- `changeguard impact` warns specifically when `match` on `Result` is replaced with `.unwrap()` in production code.
- `changeguard impact` applies +25 risk weight to error handling changes in Infrastructure directories.
- Test files are exempt from unwrap/expect warnings.
- Existing commands remain unaffected.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M17 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates observability_patterns with ERROR_HANDLE entries for fixture repos
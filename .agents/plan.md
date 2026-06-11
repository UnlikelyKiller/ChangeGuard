# Plan: Milestone Z Codex Review Remediation Tracks (Z-R1 through Z-R4)

## Overview

This plan creates four new conductor tracks to address the P1–P8 and medium/low findings from the Codex review of Milestone Z (tracks Z2–Z6). The tracks follow the established conductor format (spec + plan per track) and are designed to be robust, regression-safe, and aligned with industry best practices: parameterized queries over string interpolation, single-responsibility methods over redundant writes, shared modules over duplication, and exact assertions over substring matching.

---

## Track Z-R1: Cargo.lock Disambiguation & Schema Hardening

### Objective
Close the test-coverage gap for the source-matching version-disambiguation heuristic in `phase_cargo_dependencies`, harden the parser, and cover git/path dependency edge cases.

### Why This Matters
The most complex logic in Z4—the `PkgInfo` source-matching filter that resolves ambiguous bare dependency names when multiple versions exist—has **zero test coverage**. A regression here would silently create incorrect `DependsOn` edges or skip them entirely without failing any test.

### What Will Change
- **Test-only additions** in `tests/integration/track_z4_repro.rs`:
  - `test_cargo_lock_version_disambiguation`: Creates a lockfile with two `regex` versions (1.0 and 2.0) and a `consumer` that depends on `regex`. Asserts the edge lands on the version whose `source` matches the parent crate’s source (registry).
  - `test_cargo_lock_git_dependency`: Creates a lockfile with a git-sourced dependency and a path dependency. Asserts edges are created and node metadata captures `source`.
- **Low-risk parser hardening**: Add typed `CargoLockPackage` deserialization alongside the existing `serde_json::Value` path. If typed deserialization succeeds, use it; fall back to `Value` for forward compatibility. This does not change runtime behavior but adds schema-drift detection.

### Definition of Done
- `cargo nextest run --test integration` passes.
- New tests fail if the source-matching heuristic is inverted or removed.
- No changes to production parsing behavior on standard lockfiles.

---

## Track Z-R2: Ledger Adopt Path Deduplication & Defense-in-Depth

### Objective
Eliminate the redundant Knowledge Graph write in `execute_ledger_adopt`, centralize synthetic-entity filtering, and harden `get_transaction_files` against non-path entities.

### Why This Matters
The adopt path calls `commit_change` (which writes KG nodes/edges) and then calls `write_ledger_graph_edges` (which writes the same node + edges again). This is architectural drift: two code paths with overlapping responsibilities. If `insert_nodes` ever changes from upsert to insert-or-fail, the second call will break.

### What Will Change
- **Add `changed_files: Option<Vec<String>>` to `CommitRequest`** (`src/ledger/transaction.rs`).
- **Modify `TransactionManager::commit_change`**:
  - If `changed_files` is `Some`, use it directly for the KG `Affects` edge loop instead of calling `get_transaction_files`.
  - Apply the existing synthetic-entity filter (`drift_adoption:` / UUID check) to the override list.
- **Modify `execute_ledger_adopt`** (`src/commands/ledger/maintenance.rs`):
  - Populate `CommitRequest.changed_files` with the deduplicated real file list gathered before commit.
  - Remove the post-commit `write_ledger_graph_edges` call. Drop `tx_mgr` and move on.
- **Add synthetic-entity filtering to `write_ledger_graph_edges`** for defense-in-depth (in case future callers use it).
- **Harden `get_transaction_files`**:
  - Before inserting `tx.entity_normalized` into the file set, validate it looks like a file path (contains `/` or `.`). Skip it if it matches synthetic patterns (`drift_adoption:`, UUID).

### Definition of Done
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo nextest run --test integration` passes (including `test_adopt_writes_kg_edges_with_real_files`).
- `ledger adopt --all` produces exactly one `LedgerTransaction` node and one `Affects` edge per real file in CozoDB.
- No behavioral change for normal `ledger commit` paths.

---

## Track Z-R3: Env Schema Completeness & Regex Consolidation

### Objective
Wire the currently dead `#[allow(dead_code)]` regexes, expand coverage for real-world env-var access patterns, deduplicate regex definitions across modules, and make the reference-replacement transaction atomic.

### Why This Matters
Six compiled regexes in `env_schema.rs` are dead code. Meanwhile, common patterns like `option_env!("KEY")`, `os.environ['KEY']`, and `import.meta.env.VAR` are silently ignored, causing false negatives in `config diff`. The same regexes are duplicated in `runtime_usage.rs`, creating a maintenance hazard.

### What Will Change
- **Expand `EnvReferenceKind`** to include `ReadWithDefault` and `Write`.
- **Wire dead regexes** into `extract_references_from_source`:
  - `RUST_ENV_VAR_DEFAULT` → `ReadWithDefault`
  - `RUST_SET_ENV` → `Write`
  - `TS_ENV_DEFAULT` → `ReadWithDefault`
  - `TS_SET_ENV` → `Write`
  - `PY_ENV_GET_DEFAULT` → `ReadWithDefault`
- **Add new regexes** for uncovered patterns:
  - Rust: `std::env::var_os(…)`, `option_env!(…)`
  - Python: `os.environ['KEY']`, `os.environ.get(…)` with `from os import environ`
  - JS/TS: `import.meta.env.VAR` (Vite), destructuring `const { VAR } = process.env`
- **Consolidate regexes**: Move all `LazyLock<Regex>` definitions from `env_schema.rs` and `runtime_usage.rs` into a new `src/index/env_patterns.rs` module. Re-export from both modules. This is a pure refactor with no behavior change.
- **Atomic cleanup**: Wrap the orphan `DELETE` and per-file `INSERT`s in `EnvSchemaIndexer::extract()` inside a single SQLite `Transaction`.

### Definition of Done
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo nextest run --test integration` passes.
- `config diff` no longer reports false negatives for `option_env!` or `import.meta.env` usage.
- Deleting a file and re-indexing leaves zero orphaned `env_references` rows (verified by test).

---

## Track Z-R4: CozoDB Parameterized Queries & Test Precision

### Objective
Eliminate `format!`-based Datalog query construction (injection risk, quoting bugs), strengthen Z2/Z5/Z6 test assertions to exact-match instead of substring/loose matching, and add missing `--json` coverage.

### Why This Matters
Tests and production code interpolate URNs and strings directly into Datalog queries via `format!`. A file path containing a single quote (`src/api's.rs`) would produce invalid Datalog and panic. `run_script_with_params` exists but is unused in the test suite.

### What Will Change
- **Add safe query helpers to `src/state/storage_cozo.rs`**:
  - `query_nodes_by_category(&self, category: &str) -> Result<NamedRows>`
  - `query_edges_by_source(&self, source: &str, relation: &str) -> Result<NamedRows>`
  - `query_edges_by_target(&self, target: &str, relation: &str) -> Result<NamedRows>`
  - These use `run_script_with_params` with `:param` bindings, eliminating string interpolation.
- **Migrate tests** in `tests/integration/track_z*_repro.rs` and `tests/integration/ledger_graph_edges.rs` to use the new helpers or inline parameterized queries.
- **Strengthen assertions**:
  - Z5: Assert exact label/URN suffixes (`test_add` → `add`, not `contains("test_add")` and `contains("add")`).
  - Z6: Assert exact target URN `urn:changeguard:file:test.rs` instead of `contains("test.rs")`.
  - Z2: Add `test_data_models_impact_json_output` that asserts valid JSON structure with `impacted` array.
- **Production hardening**: Migrate the Cedar child-node cleanup in `src/index/graph_loader.rs` (lines 1250–1266) from `format!` to `run_script_with_params`.

### Definition of Done
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo nextest run --test integration` passes.
- All new and migrated CozoDB queries use parameter binding; zero `format!` interpolation into Datalog remains in tests.
- A test with a path containing special characters (e.g., `src/test-file.rs`) still passes.

---

## Rollback & Regression Strategy

1. **Track Z-R1 is test-only**: If any new test is flaky, it can be `#[ignore]`-d without affecting production.
2. **Track Z-R2 keeps signatures backward-compatible**: `CommitRequest` gets a new optional field; existing callers compile unchanged.
3. **Track Z-R3 is behavior-preserving for existing patterns**: New regexes only add coverage; consolidated regexes are re-exports.
4. **Track Z-R4 is additive**: New helpers are pure additions; old `format!` paths are removed only after tests pass.
5. **Per-track verification order**:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo nextest run --lib --bins --workspace`
   - `cargo nextest run --test integration`
   - `cargo install --path .`

---

## Files to Create / Modify

### New Files
- `conductor/trackZ-R1/spec.md`
- `conductor/trackZ-R1/plan.md`
- `conductor/trackZ-R2/spec.md`
- `conductor/trackZ-R2/plan.md`
- `conductor/trackZ-R3/spec.md`
- `conductor/trackZ-R3/plan.md`
- `conductor/trackZ-R4/spec.md`
- `conductor/trackZ-R4/plan.md`
- `src/index/env_patterns.rs` (Z-R3)

### Modified Files
- `conductor/conductor.md` (add Z-R1..Z-R4 entries)
- `tests/integration/track_z4_repro.rs` (Z-R1)
- `src/index/graph_loader.rs` (Z-R1 typed parsing, Z-R4 parameterized queries)
- `src/ledger/transaction.rs` (Z-R2)
- `src/commands/ledger/maintenance.rs` (Z-R2)
- `src/index/env_schema.rs` (Z-R3)
- `src/index/runtime_usage.rs` (Z-R3)
- `src/state/storage_cozo.rs` (Z-R4)
- `tests/integration/track_z2_repro.rs` (Z-R4)
- `tests/integration/track_z5_repro.rs` (Z-R4)
- `tests/integration/track_z6_repro.rs` (Z-R4)
- `tests/integration/ledger_graph_edges.rs` (Z-R4)

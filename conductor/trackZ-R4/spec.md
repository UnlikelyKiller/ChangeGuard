# Track Z-R4: CozoDB Parameterized Queries & Test Precision

**Status:** Planned
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** High

## Objective

Eliminate `format!`-based Datalog query construction in tests (and one production path), strengthen Z2/Z5/Z6 test assertions to exact-match instead of substring/loose matching, and add missing `--json` output coverage.

## Problem Statement

Tests across `track_z2_repro.rs`, `track_z5_repro.rs`, `track_z6_repro.rs`, and `ledger_graph_edges.rs` build CozoDB Datalog queries via `format!`:

```rust
format!("?[source, target, relation] := *edge{{source, target, relation}}, source = '{}'", tx_urn)
```

This has two problems:

1. **Injection / syntax errors**: `build_urn` does not escape single quotes. A path like `src/api's.rs` would produce invalid Datalog and panic.
2. **Poor practice**: `CozoStorage` already exposes `run_script_with_params`, which accepts a `BTreeMap<String, DataValue>` and binds safely via `:param` syntax. This helper is entirely unused in the test suite.

Additionally, tests use loose assertions (`contains("test_add")`, `contains("test.rs")`) that would pass on collisions or substrings. The Z2 test suite has no coverage for the `--json` output path.

## Acceptance Criteria

1. **Parameterized queries**: All CozoDB queries in integration tests use `run_script_with_params` with `:param` bindings instead of `format!` interpolation.
2. **Production hardening**: The Cedar child-node cleanup in `src/index/graph_loader.rs` (lines 1250–1266) migrates from `format!` to `run_script_with_params`.
3. **Exact assertions**:
   - Z5: Assert exact URN suffixes (`test_add` → `add`, not `contains`).
   - Z6: Assert exact target URN `urn:changeguard:file:test.rs`.
4. **JSON coverage**: Z2 gains a `test_data_models_impact_json_output` that asserts valid JSON with `impacted` array.
5. **Safe helpers**: Add convenience helpers to `CozoStorage` (`query_nodes_by_category`, `query_edges_by_source`, `query_edges_by_target`) that encapsulate the parameterized pattern.

## Key Files

- `src/state/storage_cozo.rs` — New safe query helpers.
- `src/index/graph_loader.rs` — Cedar cleanup parameterized.
- `tests/integration/track_z2_repro.rs` — JSON test, remove `format!`.
- `tests/integration/track_z5_repro.rs` — Exact assertions.
- `tests/integration/track_z6_repro.rs` — Exact assertions.
- `tests/integration/ledger_graph_edges.rs` — Parameterized queries.

## Definition of Done

- `cargo nextest run --lib --bins --workspace` passes.
- `cargo nextest run --test integration` passes.
- All new and migrated CozoDB queries use parameter binding; zero `format!` interpolation into Datalog remains in tests.
- A test with a path containing special characters still passes.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean.

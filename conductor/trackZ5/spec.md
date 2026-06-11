# Track Z5: Test Mapping Graph Loader
 
**Status:** In Progress
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** High

## Objective

Load test mappings from the SQLite `test_mapping` table into CozoDB during `changeguard index --analyze-graph` so that the `tests <entity>` command returns the correct mappings.

## Problem Statement

When the user runs `changeguard tests <entity>`, it reports `"No test mappings found for '<entity>'"` even though 852 mappings exist in the SQLite `test_mapping` database. The graph loader never queries this table and loads it as nodes/edges in CozoDB.

## Acceptance Criteria

1. During `changeguard index --analyze-graph`, the graph loader reads rows from the SQLite `test_mapping` table.
2. For each row:
   - It inserts a `NodeKind::Test` or `NodeKind::Symbol` node for the test symbol.
   - It inserts a `EdgeKind::Validates` edge from the test URN to the tested URN (symbol or file).
3. Running `changeguard tests <entity>` successfully returns the list of tests validating that entity.

## Key Files

* `src/index/graph_loader.rs` — Graph loading pipeline.
* `src/commands/test_mapping.rs` — `execute_tests_for_entity()` query verification.

## Definition of Done

* `cargo nextest run --lib --bins --workspace` passes.
* Verifying `tests <entity>` returns valid tests on a workspace with test mapping populated.

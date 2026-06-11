# Track Z-R1: Cargo.lock Disambiguation & Schema Hardening

**Status:** Planned
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** High

## Objective

Close the test-coverage gap for the source-matching version-disambiguation heuristic in `phase_cargo_dependencies`, harden the parser against schema drift, and cover git/path dependency edge cases.

## Problem Statement

The Z4 implementation introduced a `PkgInfo { version, source }` struct with a source-matching heuristic that resolves ambiguous bare dependency names when multiple versions of the same crate exist in the lockfile. This is the most complex logic in the Cargo.lock ingestion path, yet it has **zero test coverage**:

- Neither `test_cargo_lock_ingestion` nor `test_cargo_lock_edges` creates a `Cargo.lock` with multiple versions of the same crate.
- A regression (e.g., inverting the `==` match, or skipping ambiguous edges entirely) would not break any test.
- The lockfile is parsed into weakly typed `serde_json::Value`, meaning a future Cargo schema change would silently yield empty results instead of failing fast.
- Git and path dependencies (which may omit the `source` field or use non-registry sources) are untested.

## Acceptance Criteria

1. **Duplicate-version disambiguation test**: A new integration test creates a `Cargo.lock` with two versions of `regex` (1.0.0 and 2.0.0, both registry-sourced) and a `consumer` crate depending on `regex`. After indexing, the `DependsOn` edge from `consumer` must target exactly one of the `regex` nodes, and that node must have a matching `source`.
2. **Git/path dependency test**: A new integration test creates a `Cargo.lock` with a git-sourced dependency and a path dependency. After indexing, nodes exist with correct metadata capturing `source`.
3. **Parser hardening**: Add an optional strongly typed `CargoLockFile` / `CargoLockPackage` deserialization path alongside the existing `serde_json::Value` fallback. If typed deserialization succeeds, use it; otherwise fall back. This preserves forward compatibility while adding schema-drift detection.
4. **Zero production behavior change** on standard lockfiles.

## Key Files

- `tests/integration/track_z4_repro.rs` — New tests.
- `src/index/graph_loader.rs` — Parser hardening in `phase_cargo_dependencies`.

## Definition of Done

- `cargo nextest run --lib --bins --workspace` passes.
- `cargo nextest run --test integration` passes.
- New tests fail if the source-matching heuristic is inverted, removed, or incorrectly skips edges.
- No changes to production parsing behavior on standard lockfiles.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean.

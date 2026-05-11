# Track 53-1: Storage Infrastructure Stabilization (CozoDB Fork Integration)

## Objective
Stabilize the Knowledge Graph and Semantic Discovery infrastructure by resolving architectural limitations in the upstream CozoDB engine and fixing cross-platform path-handling bugs in ChangeGuard's synchronization engine.

## Problem Statement
1.  **Engine Fragility**: The upstream `cozo` crate has performance bottlenecks in FTS and HNSW search (serial execution) and lacks robust "in-loop" predicate filtering for semantic queries.
2.  **State Contention**: Concurrent access to the Knowledge Graph occasionally causes file-locking errors on Windows when switching between `mem` and `sled` backends dynamically.
3.  **CI Inconsistency**: Path resolution in `IncrementalSyncEngine` incorrectly uses the process working directory instead of the repository root, causing integration test failures in Linux CI environments.
4.  **HNSW Determinism**: HNSW vector search requires a fixed dimension (384), but configuration gaps allow 0-dimension vectors to be passed, crashing the storage engine.

## Scope
-   Migrate `ChangeGuard` to the `UnlikelyKiller/cozo-redux` fork via Git dependency.
-   Enforce the `sled` backend for persistent storage to ensure multi-threaded stability.
-   Implement parameterized CozoDB scripts to handle large-scale data ingestion (SQL/Datalog).
-   Fix path normalization in `IncrementalSyncEngine` for cross-platform CI compatibility.
-   Implement a runtime dimension fallback (384) in the semantic discovery layer.

## Deliverables
-   Updated `Cargo.toml` with `cozo-redux` git source.
-   Parameterized `GraphLoader` and `SemanticExtractor`.
-   Robust `IncrementalSyncEngine` with repository-relative path resolution.
-   Verified CI pipeline with `cargo deny` exceptions for the fork.

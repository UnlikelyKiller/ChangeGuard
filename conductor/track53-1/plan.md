# Track 53-1: Storage Infrastructure Stabilization (CozoDB Fork Integration) - Implementation Plan

## Phase 1: Engine Migration
1.  **Update Dependencies**: Modify `Cargo.toml` to point `cozo` to `https://github.com/UnlikelyKiller/cozo-redux`.
2.  **Lockfile Update**: Run `cargo update -p cozo` to pull Tracks 009-012 (Parallel Search, Graph Repair, PQ).
3.  **CI Hygiene**: Update `deny.toml` to allow the git source for the fork.

## Phase 2: Storage Hardening
1.  **Backend Centralization**: Force `CozoStorage` to use the `sled` engine for persistent paths, preventing locking collisions.
2.  **Parameterized Queries**: 
    - Refactor `GraphLoader::build_native_graph` to use `run_script_with_params`.
    - Refactor `SemanticExtractor::ingest_into_cozo` to use `run_script_with_params`.
3.  **Dimension Safety**: 
    - Implement a 384-dimension fallback in `SemanticDiscovery::new`.
    - Ensure `SemanticEmbedder` generates zero-vectors of the correct size.

## Phase 3: Synchronization Stability
1.  **Path Normalization**: Update `IncrementalSyncEngine` to resolve paths relative to `self.repo_path` instead of the process CWD.
2.  **Test Reliability**: 
    - Increase `watch_graph_sync` test timeout to 5 seconds.
    - Fix variable shadowing/scoping in `incremental.rs`.

## Phase 4: Verification
1.  **Library Tests**: Run `cargo test --lib` (874 tests).
2.  **Integration Tests**: Run `cargo test --test watch_graph_sync`.
3.  **Manual Triage**: Verify `changeguard index --semantic --analyze-graph` on the full repo.
4.  **End-to-End Search**: Verify `changeguard search "..."` retrieval accuracy.

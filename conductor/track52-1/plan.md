# Implementation Plan: Track 52-1 — Real-time Graph Sync (Watcher Bridge)

## Goal
Extend the `watch` command to perform incremental AST parsing and update the CozoDB Knowledge Graph in real-time, maintaining consistency with full-index semantics.

---

## Phase 1: Data Model & CozoDB Delta Primitives (TDD: Red Commit)

- [ ] **Task 1.1**: Add delta-mutation methods to `src/state/storage_cozo.rs`.
  - Implement `remove_nodes_by_id(ids: &[String]) -> Result<()>` using `:rm node { id }`.
  - Implement `remove_edges_for_source(source_ids: &[String]) -> Result<()>` using `:rm edge { source, target, relation }` with a `source IN [...]` filter (via CozoDB list binding).
  - Implement `put_node_batch(nodes: &[GraphNode]) -> Result<()>` and `put_edge_batch(edges: &[GraphEdge]) -> Result<()>` wrapping the existing JSON-batch `:put` pattern.
  - Define `GraphNode` and `GraphEdge` structs in `src/state/storage_cozo.rs` (or a new `src/state/cozo_types.rs` if preferred) to avoid leaking raw JSON tuples into `incremental.rs`.

- [ ] **Task 1.2**: Write failing unit tests for the new CozoDB delta methods.
  - `test_put_node_batch_and_query`: insert two nodes, query back by ID.
  - `test_remove_nodes_by_id`: insert nodes, remove one, assert count drops.
  - `test_remove_edges_for_source`: insert edges, remove by source, assert remaining edges.
  - `test_idempotent_put`: run `:put` twice with the same node; assert no duplicates.

- [ ] **Task 1.3**: Update `src/index/orchestrator.rs` to expose helper methods needed by the sync engine.
  - Make `delete_file_symbols(&mut self, file_path: &str) -> Result<()>` public.
  - Make `clear_structural_edges(&self, file_ids: &[i64]) -> Result<()>` public.
  - Extract a shared `index_file_with_edges(&self, path: &Utf8Path) -> Result<(ProjectFile, Vec<ProjectSymbol>, Vec<CallEdge>)>` from `index_file` so `incremental.rs` can obtain edges without duplicating logic.

- [ ] **Task 1.4**: Write failing tests for `index_file_with_edges` parity.
  - Assert that `index_file_with_edges` returns the same symbols and edges as calling `index_file` followed by `extract_calls` manually.

---

## Phase 2: Incremental Sync Engine (TDD: Red → Green)

- [ ] **Task 2.1**: Create `src/index/incremental.rs` with core types.
  - `pub struct IncrementalSyncEngine { indexer: ProjectIndexer, cozo: CozoStorage, repo_path: Utf8PathBuf }`
  - `pub struct SyncDelta { files_processed, nodes_added, nodes_removed, edges_added, edges_removed }`
  - `pub enum SyncEventKind { Created, Modified, Deleted, Renamed }`

- [ ] **Task 2.2**: Implement SQLite-side batch processing.
  - `fn apply_sqlite_delta(&mut self, events: &[WatchEvent]) -> Result<Vec<AffectedFileRecord>>`
  - For each `Create`/`Modify`: call `index_file_with_edges`, upsert `project_files`, delete old symbols, insert new symbols, return the old qualified names + new rows.
  - For each `Delete`: mark file `DELETED`, delete symbols/edges, return old qualified names.
  - Wrap the entire SQLite phase in a single `unchecked_transaction` via `StorageManager`.

- [ ] **Task 2.3**: Implement CozoDB-side delta processing.
  - `fn apply_cozo_delta(&self, affected: &[AffectedFileRecord]) -> Result<SyncDelta>`
  - Compute the set of old node IDs (file paths + old qualified names) to remove.
  - Compute the set of old source IDs for edge removal.
  - Build new `GraphNode` list (file nodes + symbol nodes) and `GraphEdge` list from `CallEdge`.
  - Batch execute `:rm` followed by `:put` in a single CozoDB script (or two sequential scripts if CozoDB scoping requires it).

- [ ] **Task 2.4**: Implement top-level `process_batch` orchestration.
  - Filter events to supported extensions (`SUPPORTED_EXTENSIONS`) and non-binary.
  - Deduplicate events by path (last event wins) to handle rapid-fire modifications.
  - Call `apply_sqlite_delta` then `apply_cozo_delta`.
  - Return `SyncDelta`.
  - Log summary at `info!` level.

- [ ] **Task 2.5**: Write the green-commit tests for `IncrementalSyncEngine`.
  - `test_process_batch_modify_one_file`: synthetic repo, modify `src/lib.rs`, assert delta counts and CozoDB node label updated.
  - `test_process_batch_delete_one_file`: delete `src/lib.rs`, assert node and edge removal.
  - `test_process_batch_parse_failure_skips_file`: corrupt one file in a 2-file batch; assert the good file still syncs and the bad file is warned.
  - `test_process_batch_no_cozo_graceful`: run with `cozo: None` equivalent; assert SQLite still updates and no panic.

---

## Phase 3: Watcher Bridge Integration

- [ ] **Task 3.1**: Refactor `src/commands/watch.rs` to invoke the sync engine.
  - Inside the `WatchBatch` callback, after drift detection, initialize `IncrementalSyncEngine` using the existing `StorageManager` and `CozoStorage` instances.
  - Call `engine.process_batch(&batch)` and log the resulting `SyncDelta`.
  - On error, emit `tracing::warn!("Incremental graph sync failed: {}", err)` and continue the watcher loop.

- [ ] **Task 3.2**: Add `--no-graph-sync` flag to `WatchArgs` (in `src/cli.rs` if applicable, or local CLI parser).
  - When provided, skip the `IncrementalSyncEngine` invocation to allow users to disable live KG updates.

- [ ] **Task 3.3**: Update `src/index/mod.rs` to export `incremental` module.
  - Add `pub mod incremental;`.

- [ ] **Task 3.4**: Write integration test for the watcher callback.
  - `tests/watch_graph_sync.rs`: create a temp repo, start a `Watcher`, write a `.rs` file, receive the batch, assert CozoDB contains the expected file node within 2 seconds.

---

## Phase 4: Consistency Verification & Hardening

- [ ] **Task 4.1**: Write the graph-consistency integration test.
  - `tests/incremental_graph_consistency.rs`:
    1. Clone a fixture repo (or use inline fixtures).
    2. Run `ProjectIndexer::full_index` + `build_native_graph` into CozoDB A.
    3. Apply 5 random file mutations via `IncrementalSyncEngine` into CozoDB A.
    4. Run `ProjectIndexer::full_index` + `build_native_graph` into a fresh CozoDB B.
    5. Assert `node_count()`, `edge_count()`, and a reachability query match exactly.

- [ ] **Task 4.2**: Audit for forbidden patterns.
  - Run `rg "\.unwrap\(\)|\.expect\(" src/index/incremental.rs src/commands/watch.rs src/state/storage_cozo.rs` and confirm zero matches in production code (tests exempt).
  - Verify all fallible paths return `miette::Result`.

- [ ] **Task 4.3**: Performance smoke test.
  - Create a benchmark-like test that processes a batch of 10 modified files and asserts elapsed time < 500 ms (use `std::time::Instant`; mark with `#[ignore]` if too flaky for CI).

- [ ] **Task 4.4**: Final CI gate.
  - Run `cargo fmt --all -- --check`.
  - Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - Run `cargo test --workspace`.
  - Address all warnings and test failures.

---

## Definition of Done (DoD)

- [ ] `src/index/incremental.rs` exists and compiles with zero warnings.
- [ ] `src/state/storage_cozo.rs` exposes batched `:put` and `:rm` helpers.
- [ ] `src/commands/watch.rs` invokes the incremental sync engine per debounced batch.
- [ ] Graph consistency test passes: incremental mutations ≡ full re-index.
- [ ] Zero `unwrap()` / `expect()` in production code for this track.
- [ ] All module boundaries respected (CLI → `commands`, parsing → `index`, persistence → `state`).
- [ ] `cargo test`, `cargo fmt`, and `cargo clippy` pass cleanly.

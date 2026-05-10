# Technical Specification: Track 52-1 — Real-time Graph Sync (Watcher Bridge)

## 1. Objective
Convert the Knowledge Graph from a batch-processed index into a live-updating state by bridging the `watch` command with incremental AST parsing and targeted CozoDB mutations.

## 2. Context
Currently, the Knowledge Graph is refreshed via `changeguard index`, which performs a full or incremental scan of the repository and rebuilds the CozoDB `node` and `edge` relations from SQLite. For downstream live-visualization (Track 52-2) and real-time impact analysis, the graph must reflect file-system changes as they happen on disk without requiring a manual re-index.

## 3. Requirements

### 3.1 Functional Requirements
1. **Incremental Parser Interface** (`src/index/incremental.rs`)
   - Accept a `WatchBatch` (or single `WatchEvent`) and parse only the affected files using the existing Tree-sitter infrastructure.
   - Reuse `ProjectIndexer::index_file` and `Language`-specific extractors (`parse_symbols`, `extract_calls`) to produce `ProjectFile` and `ProjectSymbol` rows.
   - Support `Create`, `Modify`, `Delete`, and `Rename` event kinds.

2. **SQLite State Synchronization**
   - Upsert `project_files` and `project_symbols` rows for created/modified files.
   - Cascade-delete dependents (`structural_edges`, `api_routes`, `data_models`, etc.) for affected file IDs before re-insertion.
   - Mark deleted files as `parse_status = 'DELETED'` and remove their symbols.

3. **CozoDB Delta Protocol**
   - Remove stale `node` and `edge` entries scoped to the changed file paths (using qualified names derived from the old symbol set).
   - Insert new `node` entries for the file and its symbols.
   - Insert new `edge` entries for resolved structural calls extracted from the changed file.
   - Maintain graph consistency without requiring a full `build_native_graph` reload.

4. **Watcher Integration**
   - Extend the `watch` command callback in `src/commands/watch.rs` to invoke the incremental sync engine after drift detection.
   - Respect the existing debounce/batch pipeline (`Watcher` → `WatchBatch`).
   - Failures in graph sync must be logged via `tracing::warn` and must not crash the watcher loop.

### 3.2 Non-Functional Requirements
- **Latency**: End-to-end latency from file-system event to committed CozoDB delta must be < 500 ms for batches of ≤ 10 files on a local SSD.
- **Reliability**: No `unwrap()`, `expect()`, or panics in the sync path; use `thiserror` + `miette::Diagnostic` for all error propagation.
- **Local-First**: All parsing and graph updates happen offline. No network calls.
- **Windows Resilience**: All path logic uses `camino::Utf8PathBuf`; no hardcoded `/` separators in persisted identifiers.
- **Atomicity**: Each `WatchBatch` must result in a single SQLite transaction and a single CozoDB script execution for the batch.

## 4. API Contracts

### 4.1 Watch Event Handling
```rust
// src/index/incremental.rs
pub struct IncrementalSyncEngine {
    indexer: ProjectIndexer,
    cozo: CozoStorage,
}

impl IncrementalSyncEngine {
    pub fn process_batch(&mut self, batch: &WatchBatch) -> Result<SyncDelta>;
}

pub struct SyncDelta {
    pub files_processed: usize,
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
}
```
- **Input**: `WatchBatch` containing 1..N `WatchEvent`s with `Utf8PathBuf` paths and `WatchEventKind`.
- **Output**: `SyncDelta` summarizing the mutation applied to CozoDB.
- **Error Policy**: If any file in the batch fails to parse, the error is recorded, the file is skipped, and the rest of the batch continues.

### 4.2 Incremental Parser Interface
```rust
// src/index/incremental.rs (private helper)
fn parse_and_extract(
    indexer: &ProjectIndexer,
    path: &Utf8Path,
) -> Result<(ProjectFile, Vec<ProjectSymbol>, Vec<CallEdge>)>;
```
- Delegates symbol extraction to `crate::index::languages::parse_symbols`.
- Delegates call-edge extraction to `crate::index::languages::extract_calls`.
- Returns complexity-scored symbols identical to `ProjectIndexer::index_file`.

### 4.3 CozoDB Update Protocol
```rust
// src/state/storage_cozo.rs
impl CozoStorage {
    pub fn remove_nodes_by_id(&self, ids: &[String]) -> Result<()>;
    pub fn remove_edges_for_source(&self, source_ids: &[String]) -> Result<()>;
    pub fn put_node_batch(&self, nodes: &[GraphNode]) -> Result<()>;
    pub fn put_edge_batch(&self, edges: &[GraphEdge]) -> Result<()>;
}
```
- **Deletion**: Uses `:rm` on `node { id }` and `:rm` on `edge { source, target, relation }` scoped to the affected qualified names.
- **Insertion**: Uses `:put node` and `:put edge` with batched JSON arrays (same pattern as `build_native_graph`).
- **Idempotency**: Running the same delta twice must yield the same final graph state (upsert semantics via `:put`).

## 5. Testing Strategy

### 5.1 Unit Tests
- **Synthetic Batch Processing** (`src/index/incremental.rs` tests):
  - Create a temp repo with 3 Rust files and a pre-seeded SQLite + CozoDB state.
  - Emit a `WatchBatch` modifying one file; assert `SyncDelta` counts match expectations.
  - Emit a `WatchBatch` deleting one file; assert nodes/edges are removed.
- **Parser Delegation**:
  - Assert that `parse_and_extract` returns the same `ProjectSymbol` set as `ProjectIndexer::index_file` for a given fixture.

### 5.2 Integration Tests
- **Graph Consistency** (`tests/incremental_graph_consistency.rs`):
  - Seed a graph from a fixture repo via `full_index` + `build_native_graph`.
  - Perform 5 random file edits through `IncrementalSyncEngine`.
  - Run `build_native_graph` again into a fresh CozoDB instance.
  - Assert node counts, edge counts, and specific reachability queries are identical.
- **Watch Command E2E** (`tests/watch_graph_sync.rs`):
  - Spawn a `Watcher` on a temp directory with a callback that invokes `IncrementalSyncEngine`.
  - Write a file, wait for the batch, and query CozoDB to confirm the node exists.

### 5.3 Negative Tests
- **Parse Failure Resilience**: Corrupt a file in the batch; assert the batch still commits for the other files and the corrupt file is logged.
- **Missing CozoDB**: Run `process_batch` when `cozo` is `None`; assert graceful no-op with warning.
- **Rapid Fire**: Trigger 20 rapid writes to the same file; assert final graph state reflects the last successful parse (debounce handles deduplication).

## 6. Dependencies & Risks

| Dependency | Status | Impact |
|---|---|---|
| Track G6 (Native Structural Extraction) | Completed | Required for `extract_calls` and qualified names. |
| Track G1/G2 (CozoDB Schema) | Completed | Required for `node`/`edge` relation definitions. |
| Track L2-1 (Drift Detection) | Completed | Watcher loop already established; we extend the callback. |
| `notify_debouncer_full` | Existing | Provides batching; no changes needed. |
| `tree-sitter` + language crates | Existing | Used for incremental parsing. |

### Risks & Mitigations
- **CozoDB `:rm` Performance**: Removing edges by source ID on a very large graph may be slow. **Mitigation**: Scope deletions to the exact set of qualified names derived from the old `project_symbols` rows for the affected file, and batch all `:rm` commands into a single script per batch.
- **File Read Races**: A file may be mid-write when the event fires. **Mitigation**: Reuse existing debounce; add a single retry with 50 ms backoff if `fs::read_to_string` returns an IO error.
- **State Directory Feedback Loop**: If the watcher observes `.changeguard/` or `output/` files, it could trigger recursive indexing. **Mitigation**: Existing `ignore_patterns` in `Config::watch` already exclude these; ensure the sync engine never overrides them.
- **Schema Drift**: If the CozoDB `node`/`edge` schema changes in a future track, the delta protocol must be updated. **Mitigation**: Centralize schema-aware serialization in `storage_cozo.rs` so `incremental.rs` only deals with domain structs.

## 7. Success Criteria
- [ ] Modifying a source file while `changeguard watch` is running updates the CozoDB KG within **< 500 ms**.
- [ ] Deleting a source file removes its corresponding `node` and `edge` entries from CozoDB.
- [ ] After **N** incremental updates, running a full `build_native_graph` produces an identical graph (node count, edge count, and reachability queries).
- [ ] Zero usage of `unwrap()` or `expect()` in `src/index/incremental.rs` and all touched watcher code.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --workspace` pass cleanly.
- [ ] The implementation respects module boundaries: CLI logic in `src/commands/watch.rs`, parsing in `src/index/`, persistence in `src/state/`.

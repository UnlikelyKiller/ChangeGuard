# Track RE3: Decouple `src/index/orchestrator.rs`

## Objective
Separate the worker coordination and lifecycle management from the language-specific symbol extraction and file traversal.

## Requirements
- **Worker Coordination**: Move the `rayon` and crossbeam channel management to a dedicated `src/index/worker_pool.rs`.
- **Traversal**: Move the filesystem walking and gitignore filtering to `src/index/walker.rs`.
- **Abstraction**: Use a `Job` system to represent work items (Parse, Index, Enrich).

## Definition of Done (DoD)
- [ ] `src/index/orchestrator.rs` is reduced to < 500 lines.
- [ ] Worker pool and filesystem walker are reusable components.
- [ ] `changeguard index` remains functional and performance-parity is maintained.
- [ ] Integration tests in `tests/incremental_graph_consistency.rs` pass.

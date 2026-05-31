## Plan: Project Index Decomposition
### Phase 1: Scaffolding and API Preservation
- [ ] Task 1.1: Create `src/index/orchestrator.rs`, `src/index/git_worker.rs`, `src/index/ast_worker.rs`, and `src/index/graph_worker.rs`.
- [ ] Task 1.2: Move the `ProjectIndex` struct definition to `orchestrator.rs` and re-export it in `src/index/mod.rs` to keep the public API consistent for `src/commands/index.rs`.
### Phase 2: Worker Extraction
- [ ] Task 2.1: Migrate Git-related indexing logic into `git_worker.rs`.
- [ ] Task 2.2: Migrate AST extraction logic into `ast_worker.rs`, implementing parallel iteration (`rayon`) for safe parts.
- [ ] Task 2.3: Migrate Graph insertion logic into `graph_worker.rs`, ensuring thread-safe access for parallel execution if applicable.
### Phase 3: Orchestration and Concurrency Optimization
- [ ] Task 3.1: Wire `git_worker`, `ast_worker`, and `graph_worker` together in `orchestrator.rs`.
- [ ] Task 3.2: Verify and optimize parallel execution behavior (e.g., extracting ASTs concurrently across files).
### Phase 4: Testing and Cleanup
- [ ] Task 4.1: Delete `src/index/project_index.rs` and clean up `src/index/mod.rs`.
- [ ] Task 4.2: Ensure all index tests pass. Audit for zero `unwrap()` and ensure `miette` error types are preserved.
# Plan: Track RE3 (Decouple `src/index/orchestrator.rs`)

- [ ] 1. Create `src/index/worker_pool.rs` and `src/index/walker.rs`.
- [ ] 2. Move file walking logic and `ignore` crate integration to `walker.rs`.
- [ ] 3. Move the parallel execution and error aggregation logic to `worker_pool.rs`.
- [ ] 4. Define `IndexingJob` and `WorkerState` types.
- [ ] 5. Refactor the `IndexOrchestrator` to coordinate between the walker and the pool.
- [ ] 6. Ensure the progress bar and logging remain accurate during the decoupled execution.

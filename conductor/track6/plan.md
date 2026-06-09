## Plan: Track 6 - Watch Mode and Batch Debouncing
### Phase 1: Setup and Event Filtering
- [ ] Task 1.1: Add `notify-debouncer-full` (version 0.7.0) and `ignore` (or `globset`) to `Cargo.toml`.
- [ ] Task 1.2: Implement `watch::filters` module to explicitly ignore build artifacts (`target/`, `node_modules/`), temp files, and the `.changeguard/` directory itself.
- [ ] Task 1.3: Write TDD unit tests for event filtering logic, ensuring correct identification of allowed versus ignored paths.

### Phase 2: Watcher Initialization and Debouncing Strategy
- [ ] Task 2.1: Initialize `notify-debouncer-full` in `watch::debounce` with a configurable or default timeout (e.g., 500ms).
- [ ] Task 2.2: Implement the core event accumulation loop, deduplicating multiple filesystem events for the same file within a single debounce window.
- [ ] Task 2.3: Implement Windows-specific rename/save pattern handling (matching temp-file-rename atomic saves).
- [ ] Task 2.4: Write TDD integration tests using a tempdir to simulate rapid edits, renames, and branch checkout churn. Verify via `cargo test -j 1`.

### Phase 3: Batch Persistence and Integration
- [ ] Task 3.1: Define the batch state schema in `state::layout` or `watch::batch` representing accumulated filesystem events.
- [ ] Task 3.2: Implement atomic writing of the accumulated batch to `.changeguard/state/current-batch.json`.
- [ ] Task 3.3: Implement the `watch` CLI command in `commands::watch.rs` to start the watcher and log batch creations.
- [ ] Task 3.4: Write tests to ensure deleted files during the batching phase are correctly represented or filtered before saving the batch. Verify via `cargo test -j 1`.

### Phase 4: Error Handling and Robustness
- [ ] Task 4.1: Ensure all watcher errors are wrapped in idiomatic `miette::Diagnostic` error types.
- [ ] Task 4.2: Verify that watcher recovers or gracefully exits on catastrophic directory removal or file system unmounts.
- [ ] Task 4.3: Final full-suite TDD verification via `cargo test -j 1`.

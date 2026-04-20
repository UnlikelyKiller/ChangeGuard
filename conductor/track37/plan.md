## Plan: Track 37 - LSP Daemon Functional Completion

### Phase 1: SQLite & State Hardening
- [ ] Task 1.1: Edit `src/daemon/state.rs` to remove the `PRAGMA journal_mode=WAL;` execution from `ReadOnlyStorage::get_connection()`.
- [ ] Task 1.2: Update `ReadOnlyStorage` and `LspHandlers` to surface `data_stale`. When `data_stale` is true, inject a synthetic Diagnostic warning into the `publish_diagnostics` payload indicating the database is busy and data may be stale.

### Phase 2: Real-Time Analysis & Diagnostics
- [ ] Task 2.1: Modify `src/daemon/handlers.rs` `trigger_analysis` to compute current complexity using `NativeComplexityScorer` for the active document.
- [ ] Task 2.2: Combine the real-time complexity result with the cached temporal/structural data from `get_latest_packet()`.
- [ ] Task 2.3: Ensure diagnostics are published based on this combined, real-time updated state rather than purely historical data.

### Phase 3: Hover and CodeLens
- [ ] Task 3.1: Implement `LspHandlers::on_hover`. Fetch the file's impact data from the local packet/state and format a `Hover` markdown response showing dependencies and impact reasons. Remove placeholder comments.
- [ ] Task 3.2: Implement `LspHandlers::on_code_lens`. Return a `CodeLens` at range `0:0-0:0` containing a Command title with the complexity score (e.g., "Complexity: 12.5 (High)"). Remove placeholder comments.

### Phase 4: Lifecycle & Broken-Stdin Self-Termination
- [ ] Task 4.1: Update `src/daemon/lifecycle.rs` `check_stdin_alive()` (or equivalent shutdown logic) to verify parent process liveliness using `sysinfo`, ensuring the daemon exits if the parent editor crashes. Remove placeholder comments.
- [ ] Task 4.2: Ensure the daemon cleanly removes its PID file when shutting down via this termination path or the standard `shutdown` request.

### Phase 5: Testing & Validation
- [ ] Task 5.1: Create `tests/daemon_lifecycle.rs`.
- [ ] Task 5.2: Write tests for PID file creation, stale PID replacement, and cleanup.
- [ ] Task 5.3: Write tests for `ReadOnlyStorage` retry logic and `data_stale` handling.
- [ ] Task 5.4: Write tests simulating Hover and CodeLens requests to ensure they return populated `Some(...)` responses.
- [ ] Task 5.5: Run `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings` to ensure no regression in formatting or lints.

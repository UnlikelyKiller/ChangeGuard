## Plan: Track 8 Determinism Contract and Subprocess Control
### Phase 1: Execution Boundary and Safe Capture
- [ ] Task 1.1: Create `src/util/process.rs` to serve as the standardized execution boundary for all subprocesses.
- [ ] Task 1.2: Define `ExecutionResult` and `CommandOptions` data structures for configuring limits and holding outputs.
- [ ] Task 1.3: Implement basic process spawning with bounded `stdout` and `stderr` capture. Ensure truncation occurs if `max_output_bytes` is exceeded.
- [ ] Task 1.4: Use `String::from_utf8_lossy` to safely handle non-UTF-8 stream bytes.
- [ ] Task 1.5: Write unit tests in `src/util/process.rs` to verify safe output capture and truncation. Run with `cargo test -j 1`.

### Phase 2: Timeout Control and Resource Constraints
- [ ] Task 2.1: Implement strict timeout controls inside `src/util/process.rs` (or `src/verify/timeouts.rs`). Use thread/channel timeouts to monitor `Child` processes.
- [ ] Task 2.2: Ensure the subprocess is forcibly killed (`child.kill()`) if the timeout duration is reached.
- [ ] Task 2.3: Define `miette`-compatible `ProcessError` types for `Timeout`, `NotFound`, and `Failed`.
- [ ] Task 2.4: Write tests simulating hung processes (using shell sleeps) to guarantee the timeout mechanism correctly aborts the child. Run with `cargo test -j 1`.

### Phase 3: Deterministic Output Sorting
- [ ] Task 3.1: Implement an application-wide rule/helper in `src/util/normalize.rs` or `src/impact/packet.rs` to deterministically sort vectors and maps.
- [ ] Task 3.2: Update the `ImpactPacket` construction logic to ensure that all captured arrays (e.g., changed files, verification results, tool outputs) are alphabetically or key-sorted before being finalized.
- [ ] Task 3.3: Add verification tests proving that randomly ordered inputs are strictly ordered in the final structures. Run with `cargo test -j 1`.

### Phase 4: Integration with Verification Runner
- [ ] Task 4.1: Update `src/verify/runner.rs` to route all external command execution through the new `src/util/process.rs` execution boundary.
- [ ] Task 4.2: Map the `ExecutionResult` outputs directly into the verification sections of the `ImpactPacket`.
- [ ] Task 4.3: Ensure that subprocess errors (like timeouts or missing binaries) are logged as graceful diagnostics rather than hard application panics.
- [ ] Task 4.4: Perform an end-to-end integration test validating the process boundary with a dummy verification plan. Run with `cargo test -j 1`.
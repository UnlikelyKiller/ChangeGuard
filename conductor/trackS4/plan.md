# Track S4 Plan: Automated SCIP Orchestration

## Phase 1: Detection & Orchestration Engine
- [ ] Create `src/scip/orchestrator.rs`.
- [ ] Implement `detect_scip_toolchain(language)` to probe for native SCIP indexers on the system `PATH` (e.g., detecting `rust-analyzer`).
- [ ] Implement `generate_scip_index(toolchain, working_dir)` to spawn the subprocess (e.g., `rust-analyzer scip .`), capture output, and handle timeouts.

## Phase 2: CLI Integration
- [ ] Add `--auto-scip` flag to the `index` subcommand in `src/cli/args.rs`.
- [ ] Update `execute_index` in `src/commands/index.rs` to route to the SCIP orchestrator when the flag is present.
- [ ] Ensure cleanup of the temporary `index.scip` file generated during the automated run so it doesn't clutter the user's workspace.

## Phase 3: Testing & Finalization
- [ ] Add unit tests for toolchain detection logic.
- [ ] Add an integration test that mocks a `rust-analyzer` binary and verifies `changeguard index --auto-scip` triggers generation and ingestion.
- [ ] Run `changeguard verify` and ensure all tests pass.

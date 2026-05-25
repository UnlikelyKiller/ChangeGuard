# Track CR6 Plan: Strong Process Validation for Viz Server Stop

## Phase 1: Implementation
- [ ] Inspect the process listing and termination logic in `src/commands/viz_server.rs` (especially Windows-specific paths).
- [ ] Refine the process parsing to check if the image name is exactly `"changeguard.exe"` or match it using a strict pattern, rather than a broad substring check on the whole output line.
- [ ] Ensure that PIDs extracted belong to the actual binary rather than shell wrappers or helper scripts.

## Phase 2: Testing & Verification
- [ ] Implement manual verification by launching a dummy process containing `"changeguard"` in its name or arguments (e.g. `notepad.exe changeguard_notes.txt`) and running `changeguard viz-server --stop`.
- [ ] Verify that only the actual `changeguard` server process is terminated, and the dummy process remains unaffected.
- [ ] Confirm no regressions in Unix/macOS process termination logic.

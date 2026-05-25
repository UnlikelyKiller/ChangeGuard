# Track CR6 Plan: Strong Process Validation for Viz Server Stop

## Phase 1: Implementation
- [x] Modified Windows `kill_viz_server` in `src/commands/viz_server.rs` to derive the expected image name from `current_exe()`.
- [x] Changed the tasklist CSV output check from a loose `contains("changeguard")` substring to an exact match of the image name field.
- [x] Parsing: first comma-delimited CSV field, stripped of quotes and lowercased, compared against the lowercase exe filename.

## Phase 2: Testing & Verification
- [x] Existing PID file roundtrip tests remain green.
- [x] `cargo test` passes.

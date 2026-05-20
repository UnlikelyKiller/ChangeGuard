# Track J10: `viz-server` CLI Wiring or Clean Removal

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard viz-server` returns "error: unrecognized subcommand 'viz-server'" even though `src/commands/viz_server.rs` exists in the codebase. The subcommand was implemented but never registered in `src/cli.rs` (or `src/main.rs`). The binary advertises zero docs for this feature and any user who reads the source code or finds a reference to `viz-server` in documentation will get a confusing error.

There are two valid resolutions:
1. **Wire it**: Add `viz-server` to the CLI and verify it works end-to-end.
2. **Remove it**: Delete `src/commands/viz_server.rs` and any dead references if the feature is not ready for use.

Decision criteria: If `viz_server.rs` implements a working WebSocket-based arc diagram server (described in the onboarding skill) that can be tested locally, wire it. If the implementation is a stub or placeholder, remove it to keep the CLI surface honest.

This track spec covers **wiring** as the primary path, with removal as the fallback.

## Implementation Path A: Wire the Subcommand

### 1. Inspect `src/commands/viz_server.rs`
- Confirm the implementation is functional (starts a server, serves the arc diagram).
- Identify the command struct (e.g., `VizServerArgs`).

### 2. `src/cli.rs` (or `src/main.rs` dispatch)
- Add `VizServer(VizServerArgs)` variant to the `Commands` enum.
- Add `Commands::VizServer(args) => commands::viz_server::execute(args, config)?` to the dispatch match.
- Add subcommand metadata: `about = "Start a WebSocket-based live arc diagram server"`.

### 3. `src/commands/mod.rs` (if applicable)
- Add `pub mod viz_server;` if not already present.

### 4. Help text and usage
- Ensure `changeguard viz-server --help` shows meaningful options (port, open-in-browser flag).
- Add `--port <PORT>` (default 7070) and `--open` (auto-open browser) if not already present.

## Implementation Path B: Remove the Dead File

If `viz_server.rs` is a stub:
- Delete `src/commands/viz_server.rs`.
- Remove any `pub mod viz_server;` from `src/commands/mod.rs`.
- Remove any dead `use` imports referencing it.
- File a note in this track's plan.md that the feature was removed pending a future implementation track.

## Success Criteria (Path A — Wiring)
- `changeguard viz-server` starts a server without error.
- `changeguard viz-server --help` shows port and open options.
- `changeguard viz-server --port 7071` starts on the alternate port.
- Server responds to HTTP GET on the configured port.
- `Ctrl+C` shuts down cleanly (no zombie processes).

## Success Criteria (Path B — Removal)
- `changeguard viz-server` returns "unrecognized subcommand" (same as now, but the dead source file is gone).
- `cargo build` produces zero unused-module warnings.
- All existing tests pass.

## Files Changed (Path A)
- `src/cli.rs` or `src/main.rs`
- `src/commands/mod.rs`
- `src/commands/viz_server.rs` (if changes needed)

## Files Changed (Path B)
- `src/commands/viz_server.rs` (deleted)
- `src/commands/mod.rs`
- Any referencing imports

## Edge Cases (Path A)
- **Port already in use**: Return a descriptive error: "Port 7070 is already in use. Use --port to specify another."
- **No graph data** (KG not indexed): Server starts but serves an empty diagram with a message "No graph data — run 'changeguard index --semantic' first."
- **`--open` on Windows**: Use `start` command (`std::process::Command::new("cmd").args(["/C", "start", url])`); on macOS use `open`; on Linux use `xdg-open`. Guard with `cfg` or runtime OS detection.
- **Graceful shutdown on Ctrl+C**: Register a `ctrlc` handler that sends a shutdown signal to the server thread; join the thread before exiting.

## Definition of Done
- **Path A**: `changeguard viz-server` starts successfully; `changeguard viz-server --help` shows options; Ctrl+C exits cleanly; CI gate passes.
- **Path B**: Dead file removed; `cargo build` has zero warnings; all tests pass; CI gate passes.
- CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.

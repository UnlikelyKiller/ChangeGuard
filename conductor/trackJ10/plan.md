# Track J10 Plan: `viz-server` CLI Wiring or Clean Removal

## Steps

### Discovery (determines path)
1. [ ] Read `src/commands/viz_server.rs` in full
2. [ ] Read `src/cli.rs` (or `src/main.rs`) to confirm viz-server is absent from CLI dispatch
3. [ ] Determine: is the implementation functional (proceed with Path A — Wire) or a stub (proceed with Path B — Remove)?
4. [ ] Document decision in a comment at the top of this plan

### Path A: Wire the Subcommand

#### Red Phase
5. [ ] Add CLI integration test: `changeguard viz-server --help` exits 0 and outputs usage info
6. [ ] Run CI gate — test expected to fail (subcommand unrecognized)

#### Green Phase
7. [ ] Add `VizServer(VizServerArgs)` variant to `Commands` enum in `src/cli.rs`
8. [ ] Add dispatch arm: `Commands::VizServer(args) => commands::viz_server::execute(args, config)?`
9. [ ] Add `pub mod viz_server;` to `src/commands/mod.rs` if missing
10. [ ] Add `--port <PORT>` (default 7070) and `--open` flag to `VizServerArgs` if not present
11. [ ] Add port-in-use error handling; add graceful Ctrl+C handler
12. [ ] Run `cargo build` — fix any type/import errors
13. [ ] Run CI gate — all tests expected to pass

#### Verification
14. [ ] `cargo install --path .` to rebuild binary
15. [ ] `changeguard viz-server --help` → shows port and open options
16. [ ] `changeguard viz-server` → server starts; HTTP GET on port 7070 responds
17. [ ] `Ctrl+C` → exits cleanly, no zombie process
18. [ ] `changeguard verify` passes

---

### Path B: Remove Dead File

#### Red Phase (N/A — removal is always green)
5. [ ] Run `cargo build` to confirm current warnings about unused module

#### Implementation
6. [ ] Delete `src/commands/viz_server.rs`
7. [ ] Remove `pub mod viz_server;` from `src/commands/mod.rs`
8. [ ] Run `cargo build` — confirm zero warnings and zero errors
9. [ ] Run CI gate — all tests pass

#### Verification
10. [ ] `changeguard viz-server` → "unrecognized subcommand 'viz-server'" (expected)
11. [ ] `changeguard --help` → viz-server not listed
12. [ ] `changeguard verify` passes

---

### Finalization (both paths)
- [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
- [ ] `changeguard ledger commit` with summary and reason

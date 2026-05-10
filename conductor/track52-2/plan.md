# Plan: Track 52-2 Live Viz (Arc Diagram & WebSocket Server)

## Phase 1: Infrastructure & Dependencies

- [ ] **Task 1.1**: Update `Cargo.toml`
  - Add `tokio-tungstenite = { version = "0.26", optional = true }`
  - Add `futures-util = { version = "0.3", optional = true }`
  - Add `viz-server = ["dep:tokio", "dep:tokio-tungstenite", "dep:futures-util"]` to `[features]`.
  - Ensure `tokio` features include `"rt-multi-thread"`, `"macros"`, `"time"`.
- [ ] **Task 1.2**: Update `src/commands/mod.rs`
  - Add `#[cfg(feature = "viz-server")] pub mod viz_server;`
- [ ] **Task 1.3**: Update `src/cli.rs`
  - Add `VizServer` variant to `Commands` enum gated by `#[cfg(feature = "viz-server")]`.
  - Arguments: `--port` (u16, default 8765), `--bind` (String, default "127.0.0.1"), `--open` (bool).
  - Wire match arm to `crate::commands::viz_server::execute_viz_server(port, bind, open)`.
- [ ] **Task 1.4**: Add `src/commands/viz_server.rs` module shell
  - Stub `pub fn execute_viz_server(port: u16, bind: String, open: bool) -> Result<()>`.
  - Validate `bind` is a loopback address (`127.0.0.1` or `::1`); return `miette::miette!(...)` otherwise.

## Phase 2: Graph Delta Query Engine (`src/state/`)

- [ ] **Task 2.1**: Create `src/state/delta.rs`
  - Define `struct GraphSnapshot { nodes: HashMap<String, VizNode>, edges: HashSet<VizEdge> }`
  - Define `struct GraphDelta { added_nodes: Vec<VizNode>, removed_nodes: Vec<String>, updated_nodes: Vec<VizNode>, added_edges: Vec<VizEdge>, removed_edges: Vec<VizEdge> }`
  - Implement `GraphSnapshot::from_cozo(cozo: &CozoStorage) -> Result<Self>`
    - Re-use the same Datalog queries as `src/commands/viz.rs` (`*node{id, label, category, risk_score}` and `*edge{source, target, relation}`).
  - Implement `GraphSnapshot::diff(&self, other: &Self) -> GraphDelta`
- [ ] **Task 2.2**: Add delta query methods to `src/state/storage_cozo.rs` (or keep in `delta.rs`)
  - Ensure `CozoStorage` is accessible from the new module.
- [ ] **Task 2.3**: Unit tests in `src/state/delta.rs`
  - Test empty snapshot diff returns empty delta.
  - Test node addition, removal, and risk_score update detection.
  - Test edge addition and removal.

## Phase 3: WebSocket Server (`src/commands/viz_server.rs`)

- [ ] **Task 3.1**: Implement `start_server` async function
  - Create `tokio::runtime::Runtime` in `execute_viz_server` and `block_on(start_server(...))`.
  - Bind TCP listener to `bind:port`.
  - Use `tokio_tungstenite::accept_async` to upgrade HTTP connections on `/ws`.
  - Serve the Arc Diagram HTML page on `GET /`.
- [ ] **Task 3.2**: Implement broadcast channel
  - Use `tokio::sync::broadcast::channel::<String>(16)` for delta distribution.
  - Spawn a background task that:
    - Polls `GraphSnapshot::from_cozo` every 250 ms while `receiver_count > 0`.
    - Compares with last snapshot; if changed, computes `GraphDelta`, serializes to JSON, and broadcasts.
    - Sends a `heartbeat` every 30 s.
- [ ] **Task 3.3**: Client connection handler
  - On connect, immediately send a `type: "snapshot"` message with the current full graph.
  - Subscribe to broadcast channel and forward messages until the client disconnects.
  - Log connection/disconnection at `tracing::info` level.
- [ ] **Task 3.4**: Unit tests for server logic
  - Extract server logic into a testable `VizServer` struct.
  - Test that binding to loopback succeeds and non-loopback fails.
  - Test broadcast channel behavior with mock clients.

## Phase 4: D3.js Arc Diagram Frontend

- [ ] **Task 4.1**: Embed or generate the HTML template
  - Option A: Inline a `const ARC_DIAGRAM_HTML: &str` in `src/commands/viz_server.rs` containing the full HTML/CSS/JS.
  - Option B: Create `templates/arc_diagram.html` and embed via `include_str!`.
  - The template must reference `ws://127.0.0.1:{{port}}/ws` (use a placeholder replaced at serve time).
- [ ] **Task 4.2**: Implement D3.js Arc Diagram
  - Horizontal node axis sorted by community, then label.
  - Arcs drawn with `d3.arc()` between node positions.
  - Node color by `community`, radius scaled by `risk_score`.
  - Transition animations on enter/update/exit using D3 data joins.
  - Click-to-focus metadata panel.
  - Search box that dims non-matching nodes.
- [ ] **Task 4.3**: WebSocket client logic in JS
  - Connect to `ws://` endpoint.
  - On `snapshot`: replace entire dataset and re-render.
  - On `delta`: apply additions/removals/updates with D3 transitions.
  - Reconnect with exponential backoff on disconnect.

## Phase 5: Integration & CLI Wiring

- [ ] **Task 5.1**: Wire `src/main.rs` / `src/cli.rs`
  - Ensure the `VizServer` command match arm is present and gated by `#[cfg(feature = "viz-server")]`.
- [ ] **Task 5.2**: Add `--open` support
  - If `--open` is true, call `webbrowser::open(url)` (add `webbrowser` as optional dep under `viz-server` if desired; otherwise print a warning).
  - If the crate is unavailable, simply print the URL in bold cyan.
- [ ] **Task 5.3**: Graceful shutdown
  - Hook `ctrlc` to signal the tokio runtime to drop the TCP listener and broadcast task.
  - Ensure all WebSocket peers receive a close frame before exit.

## Phase 6: Verification (TDD)

- [ ] **Task 6.1**: `cargo test` passes for all new modules (`delta.rs`, `viz_server.rs`).
- [ ] **Task 6.2**: `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] **Task 6.3**: `cargo fmt --all -- --check` passes.
- [ ] **Task 6.4**: Manual end-to-end test
  1. `cargo run --features viz-server -- viz-server`
  2. Open browser at printed URL.
  3. In a second terminal, run `cargo run -- watch` and edit a `.rs` file.
  4. Confirm the Arc Diagram updates within 1 s.
- [ ] **Task 6.5**: ChangeGuard hygiene checks
  - Run `changeguard ledger status --compact` before starting work and after finishing.
  - Run `changeguard verify` after implementation.

## Definition of Done

- [ ] `changeguard viz-server` command is implemented, gated behind the `viz-server` feature.
- [ ] D3.js Arc Diagram template is embedded and renders correctly in Chrome/Firefox.
- [ ] Real-time graph deltas are computed in `src/state/delta.rs` and pushed over WebSocket.
- [ ] Server binds exclusively to localhost; non-loopback addresses are rejected.
- [ ] No `unwrap()` or `expect()` in production paths; all errors use `miette::Diagnostic`.
- [ ] Module boundaries respected: CLI in `src/commands/`, graph queries in `src/state/`.
- [ ] All new code is covered by unit tests; integration test verifies WebSocket snapshot + delta delivery.
- [ ] `Cargo.toml`, `src/commands/mod.rs`, `src/cli.rs`, `src/commands/viz_server.rs`, `src/state/delta.rs` are the primary modified files.

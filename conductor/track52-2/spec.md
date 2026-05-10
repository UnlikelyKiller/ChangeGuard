# Track 52-2: Live Viz (Arc Diagram & WebSocket Server)

## Objective

Implement a local-first, interactive Arc Diagram visualization that receives real-time Knowledge Graph deltas via a WebSocket server embedded in the ChangeGuard CLI. The server binds to localhost only, polls CozoDB for graph changes, and pushes minimal delta payloads to a standalone D3.js frontend.

## Context

Track G5 introduced a static `changeguard viz` command that exports the full Knowledge Graph to a vis-network HTML file. Track 52-1 (Watcher Bridge) enables incremental updates to CozoDB as files change on disk. This track closes the loop by providing a live-updating visualization that reflects those incremental changes without requiring a page refresh or manual re-export.

The Arc Diagram is chosen because it clearly renders dense dependency graphs as arcs above a linear node axis, making cross-module call graphs and semantic edges easy to read at a glance.

## Requirements

### Functional Requirements

1. **WebSocket Server**
   - A lightweight async server running inside `changeguard` that broadcasts JSON graph deltas to connected clients.
   - Must bind to `127.0.0.1` by default (configurable port, default `8765`).
   - Must support multiple concurrent browser clients.
   - Graceful shutdown on `Ctrl+C`.

2. **Graph Delta Engine**
   - Poll CozoDB at a debounced interval (default 250 ms) while clients are connected.
   - Compute a delta between the last known snapshot and the current `node` / `edge` relations.
   - Emit minimal delta messages (`added_nodes`, `removed_nodes`, `added_edges`, `removed_edges`) rather than full snapshots on every tick.
   - Send a full snapshot on initial client connection.

3. **Arc Diagram Frontend**
   - A self-contained HTML page served by the WebSocket server (or embedded in the binary and written to a temp path).
   - Render nodes on a horizontal axis and edges as arcs above/below the axis using D3.js v7.
   - Color-code nodes by `community` (from Louvain detection) and highlight high `risk_score` nodes.
   - Smoothly animate transitions when deltas arrive: enter/exit arcs with opacity/position tweens.
   - Clicking a node shows metadata panel: `id`, `label`, `category`, `risk_score`, `community`.
   - Support search/filter by node label.

4. **CLI Integration**
   - New command: `changeguard viz-server [--port <PORT>] [--bind <BIND>]`.
   - Alternatively, extend existing `changeguard viz` with a `--live` flag that launches the server.
   - Server prints the local URL (`http://127.0.0.1:8765`) on startup.

### Non-Functional Requirements

1. **Local-First Security**: Server must reject or never bind to non-loopback interfaces. Binding to `0.0.0.0` is forbidden.
2. **Performance**: Delta computation must complete in <50 ms for graphs up to 10k nodes/edges. WebSocket latency to client <100 ms.
3. **Resource Throttling**: Polling pauses when no clients are connected.
4. **Error Handling**: All errors use `thiserror` + `miette::Diagnostic`. No `unwrap()` or `expect()` in production code.
5. **Windows Resilience**: Paths use `camino::Utf8PathBuf` where possible; avoid hardcoded Unix separators.

## API Contracts

### WebSocket Protocol

- **Transport**: `ws://127.0.0.1:<port>/ws`
- **Encoding**: Text frames containing UTF-8 JSON.
- **Client → Server**: None required (one-way push). Optional `{"action":"ping"}` for keep-alive.
- **Server → Client**:
  - `type: "snapshot"` — sent immediately on connection.
  - `type: "delta"` — sent when graph changes are detected.
  - `type: "heartbeat"` — sent every 30 s to keep NAT/firewall sessions alive.

### Delta Message Format

```json
{
  "type": "delta",
  "timestamp": "2026-05-09T12:34:56Z",
  "nodes": {
    "added": [
      { "id": "src::foo::bar", "label": "bar", "category": "function", "risk_score": 0.42, "community": 3 }
    ],
    "removed": ["src::old::dep"],
    "updated": [
      { "id": "src::foo::baz", "risk_score": 0.88 }
    ]
  },
  "edges": {
    "added": [
      { "from": "src::foo::bar", "to": "src::foo::baz", "label": "calls" }
    ],
    "removed": [
      { "from": "src::foo::old", "to": "src::foo::baz", "label": "calls" }
    ]
  }
}
```

Snapshot message uses the same schema with `type: "snapshot"` and omits `removed`/`updated` sections (full replace).

### CLI Command Interface

```
changeguard viz-server [OPTIONS]

Options:
  -p, --port <PORT>    WebSocket server port [default: 8765]
  -b, --bind <BIND>    Bind address [default: 127.0.0.1]
  -o, --open           Automatically open the browser on startup
  -h, --help           Print help
```

## Testing Strategy

1. **Unit Tests**
   - `DeltaComputer`: Feed two synthetic CozoDB result sets and assert the correct `added`/`removed`/`updated` sets are produced.
   - `WsBroadcaster`: Use a mock WebSocket client (via `tokio-tungstenite` test client) to verify snapshot is received on connect and deltas are broadcast when the graph changes.
   - `HtmlTemplate`: Verify the embedded template string contains valid D3.js arc-diagram rendering code and references the expected WebSocket URL.

2. **Integration Tests**
   - Start the viz-server on an ephemeral port, connect a test client, trigger a CozoDB mutation (via `CozoStorage::run_script`), and assert the client receives a delta within 1 second.
   - Verify binding to `127.0.0.1` succeeds and binding to `0.0.0.0` is rejected at the argument-validation layer.

3. **Manual QA**
   - Run `changeguard viz-server`, open the provided URL in Chrome/Firefox, modify a source file while `changeguard watch` is running (Track 52-1), and visually confirm the Arc Diagram updates without refresh.

## Dependencies & Risks

### New Dependencies

| Crate | Version | Purpose | Feature Gate |
|---|---|---|---|
| `tokio-tungstenite` | `0.26` | WebSocket server | `viz-server` |
| `tokio` | `1.43` (already optional) | Async runtime | `viz-server` (or extend `daemon`) |
| `futures-util` | `0.3` | Stream utilities for WebSocket broadcast | `viz-server` |

> **Note**: `tokio` is already an optional dependency gated by `daemon`. Add a new `viz-server` feature that depends on `tokio` and `tokio-tungstenite`, or extend the `daemon` feature to include `tokio-tungstenite` if architectural review prefers a single server feature.

### Risks & Mitigations

| Risk | Mitigation |
|---|---|
| **Async runtime conflicts** — the rest of the CLI is synchronous. | Spawn a dedicated `tokio::runtime` inside `execute_viz_server` and block_on. Do not infect the sync CLI with async signatures. |
| **CozoDB threading** — CozoDB `DbInstance` may not be `Send` across threads. | Keep the `CozoStorage` handle on a single async task or use a dedicated thread with a channel. Test on Windows. |
| **Large graph delta computation** | Implement diffing in-memory with `HashMap` by `id`; limit polling frequency. Future work: expose a CozoDB change-feed if available. |
| **Frontend asset size** | Embed D3.js from CDN or ship a minified inline `<script>`; do not bundle npm tooling. |

## Success Criteria

- [ ] `changeguard viz-server` starts without error and prints a reachable `http://127.0.0.1:8765` URL.
- [ ] Opening the URL in a browser renders an interactive D3.js Arc Diagram of the current Knowledge Graph.
- [ ] Connecting a second browser client receives the full snapshot immediately.
- [ ] Modifying a tracked source file (with Track 52-1 watcher active) causes a delta message to be pushed within 1 second, and the Arc Diagram animates the change.
- [ ] Server refuses to bind to non-loopback addresses (`0.0.0.0`, `::`, external IPs).
- [ ] All production code uses `Result` + `miette::Diagnostic`; zero `unwrap()` / `expect()`.
- [ ] All new modules include unit tests with >80% line coverage.

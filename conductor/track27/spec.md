# Track 27: LSP-Lite ChangeGuard Daemon (Phase 21)

## Overview
Phase 21 extends ChangeGuard into a background intelligence platform by providing real-time data overlays in IDEs (VS Code, Cursor). This is achieved via an LSP-Lite daemon using `tower-lsp-server`. The daemon responds to code queries (like `textDocument/codeLens` and `textDocument/hover`) with localized impact, risk density, and historical context pulled deterministically from ChangeGuard's internal models.

## Architecture

### 1. Feature Isolation
- **Boundary**: All daemon-specific crates (`tokio`, `tower-lsp-server`) MUST be isolated behind a new `daemon` Cargo feature.
- **Dependency Scope**: `src/daemon/` and `src/commands/daemon.rs` are feature-gated. A standard `cargo build --no-default-features` must not link Tokio.
- **Tokio Runtime**: The daemon runs on a strictly constrained multi-thread runtime (`worker_threads(2)`). Unbounded background work is prohibited.

### 2. LSP Object Mapping (`src/output/lsp.rs`)
To cleanly bridge the gap between our deterministic intelligence data (from SQLite and Git) and standard IDEs, we introduce `src/output/lsp.rs`. This acts as an Anti-Corruption Layer:
- Translates `ImpactPacket` anomalies and Complexities into `tower_lsp_server::ls_types::Diagnostic` objects.
- Maps Temporal Coupling / Risk Scores into `ls_types::CodeLens`.
- Structures historical summaries into `ls_types::Hover`.

### 3. State Management & Contention (`src/daemon/state.rs`)
- The daemon shares `.changeguard/db.sqlite3` with the synchronous CLI.
- The daemon MUST only open **read-only** connections in WAL mode.
- **SQLITE_BUSY Mitigation**: If the main CLI is writing, the daemon MUST retry with exponential backoff (100ms, 200ms, 400ms). If it fails, it returns a stale cache payload annotated with `data_stale: true`.

### 4. Lifecycle Management (`src/daemon/lifecycle.rs`)
- **PID File**: The daemon writes to `.changeguard/daemon.pid` on startup and cleans up on a clean shutdown/exit.
- **Stale PIDs**: On startup, if the PID file exists but the process is dead, the daemon replaces the file. If the process is alive, it exits gracefully with an error.
- **Orphan Detection**: If VS Code restarts without sending a `shutdown` notification, the daemon detects broken stdin and self-terminates within 5 seconds.

### 5. LSP Server Trait Implementation (`src/daemon/server.rs` & `handlers.rs`)
- Migrates from unmaintained `lsp-types` to `tower_lsp_server::ls_types`.
- Avoids `#[async_trait]` per `tower-lsp-server` v0.23.0 updates.
- Normalizes client URIs carefully. Malformed URIs (e.g., `file:///c%3A/` vs `C:\`) will be converted using `UriExt::to_file_path()` and mismatches will just log a warning without crashing.

## Fallback & Degradation Rules
- If AST parsing fails during user typing, emit provisional scores with a `complexity_warning: "AST parse incomplete"` annotation in the `Diagnostic` rather than dropping the request.
- All errors propagating up to the LSP handler must gracefully map to JSON-RPC error codes rather than panicking the daemon process.
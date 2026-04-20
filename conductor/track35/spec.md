# Track 35: LSP Daemon Resolution

## 1. Objective
Replace the temporary watch wrapper in `src/commands/daemon.rs` with a real, fully-featured LSP server in `src/daemon/` using `tower-lsp-server` and `tokio`. Address the failures identified in Audit 3 regarding Track 27.

## 2. Core Architecture
The daemon subsystem must be strictly isolated and feature-gated.
- **Entry Point**: `src/commands/daemon.rs` builds a constrained `tokio` runtime and spawns the LSP server.
- **LSP Server**: `src/daemon/server.rs` implements the `tower_lsp_server::LanguageServer` trait using native `async fn` methods (no `#[async_trait]`).
- **Feature Handlers**: `src/daemon/handlers.rs` translates LSP protocol requests (Hover, CodeLens, Diagnostics) into queries against local state.
- **Lifecycle**: `src/daemon/lifecycle.rs` manages the `.changeguard/daemon.pid` file and process health.
- **State/Database**: `src/daemon/state.rs` provides read-only WAL SQLite access with exponential backoff for `SQLITE_BUSY`.

## 3. Implementation Rules & Constraints

### 3.1 Dependencies
```toml
# In Cargo.toml
[dependencies.tokio]
version = "1.x"
features = ["rt-multi-thread", "io-std", "macros"]
optional = true

[dependencies.tower-lsp-server]
version = "0.23.0"
optional = true

[features]
default = []
daemon = ["dep:tower-lsp-server", "dep:tokio"]
```

### 3.2 LSP API Migration
- All LSP type imports MUST use: `use tower_lsp_server::ls_types::*;`
- The `LanguageServer` trait in `tower-lsp-server` v0.23.0 uses native async methods. DO NOT use `#[async_trait]`.

### 3.3 Lifecycle & Process Safety
- **Tokio Runtime**: Constrained to `tokio::runtime::Builder::new_multi_thread().worker_threads(2)`.
- **PID File**: Write `.changeguard/daemon.pid` on startup. If it exists:
  - Check if process is alive. If dead, delete stale PID and proceed.
  - If alive, abort startup with an actionable error.
- **Shutdown**: Gracefully clean up PID file within 1 second on `exit` notification.
- **Broken Stdin**: Detect closed `stdin` (VS Code restart without `shutdown` notification) and self-terminate within 5 seconds.

### 3.4 State & Database Safety
- **Read-Only**: The daemon MUST NOT write to the SQLite database.
- **WAL Mode**: Use read-only WAL mode to avoid blocking CLI writers.
- **Contention Retry**: On `SQLITE_BUSY`, retry with exponential backoff (100ms, 200ms, 400ms). If still busy, return `data_stale: true` annotation in outputs.

### 3.5 IDE Integration & Edge Cases
- **URI Normalization**: Use `tower_lsp_server::ls_types::UriExt::to_file_path()` to normalize URIs (e.g., `file:///c%3A/`). Log mismatches without crashing.
- **Diagnostics (`publishDiagnostics`)**: Push verification failures and impact rules.
- **CodeLens (`textDocument/codeLens`)**: Overlay risk and complexity scores for functions/structs in the document.
- **Hover (`textDocument/hover`)**: Display historical failure probability or logical coupling summaries for hovered symbols.

## 4. Verification Gates
- `cargo build --no-default-features` MUST pass without linking `tokio` or `tower-lsp-server`.
- `cargo clippy --all-targets --all-features -- -D warnings` MUST pass (fix `ChangedFile` import in `src/output/lsp.rs`).
- Implement `tests/daemon_lifecycle.rs` testing PID file management, stale cleanup, and normal shutdown.
- Implement tests verifying `SQLITE_BUSY` retry backoff.

# Plan: Track 35 - LSP Daemon Resolution

### Phase 1: Dependency & Scaffolding
- [ ] Task 1.1: Update `Cargo.toml` to include `tokio` as an optional dependency with `rt-multi-thread`, `io-std`, and `macros` features. Update `daemon` feature to include `dep:tokio`.
- [ ] Task 1.2: Update `src/commands/daemon.rs` to build a `tokio` runtime (`worker_threads(2)`) and launch the daemon. Gate this logic entirely behind `#[cfg(feature = "daemon")]`.
- [ ] Task 1.3: Fix `src/output/lsp.rs` tests by adding the missing `ChangedFile` import so `cargo clippy --all-targets --all-features` passes.
- [ ] Task 1.4: Provide a stub implementation of `execute_daemon` for when the `daemon` feature is disabled, which returns an actionable `miette::Result` stating the daemon feature is not enabled.

### Phase 2: Lifecycle Management
- [ ] Task 2.1: Create `src/daemon/lifecycle.rs`. Implement `DaemonLifecycle` to manage `.changeguard/daemon.pid`.
- [ ] Task 2.2: Implement stale PID detection: check if the PID in the file corresponds to a running process, and if not, clean it up and proceed.
- [ ] Task 2.3: Implement broken stdin detection to forcefully self-terminate within 5 seconds if the LSP client drops the connection ungracefully.

### Phase 3: State & Database Layer
- [ ] Task 3.1: Create `src/daemon/state.rs`. Implement a read-only SQLite connection manager.
- [ ] Task 3.2: Configure the SQLite connection for WAL mode.
- [ ] Task 3.3: Implement a backoff mechanism for `SQLITE_BUSY` (100ms, 200ms, 400ms retry sequence).
- [ ] Task 3.4: Wrap queries to return a struct indicating whether the data returned is stale (`data_stale: true`) due to lock exhaustion.

### Phase 4: LSP Server Implementation
- [ ] Task 4.1: Create `src/daemon/mod.rs` and `src/daemon/server.rs`.
- [ ] Task 4.2: Implement the `tower_lsp_server::LanguageServer` trait (without `#[async_trait]`) on a `Backend` struct.
- [ ] Task 4.3: Implement `initialize` to configure server capabilities (Hover, CodeLens, TextDocumentSync, etc.).
- [ ] Task 4.4: Implement `shutdown` and `exit` hooks to properly release SQLite connections and delete the PID file.

### Phase 5: Feature Handlers (Diagnostics, Hover, CodeLens)
- [ ] Task 5.1: Create `src/daemon/handlers.rs`. Implement robust URI normalization using `tower_lsp_server::ls_types::UriExt::to_file_path()`.
- [ ] Task 5.2: Implement `didOpen`, `didChange`, and `didSave` to trigger background analysis (re-using Phase 1 watch logic but strictly for diagnostic reporting). Send diagnostics via `publishDiagnostics`.
- [ ] Task 5.3: Implement `textDocument/codeLens` to provide risk/complexity scores for functions in the currently open file.
- [ ] Task 5.4: Implement `textDocument/hover` to provide impact summaries for symbols (e.g., temporal coupling warnings or probabilistic verification stats).

### Phase 6: Testing & Quality Assurance
- [ ] Task 6.1: Create `tests/daemon_lifecycle.rs` (gated behind `#[cfg(feature = "daemon")]`).
- [ ] Task 6.2: Write tests for PID creation, stale PID removal, and graceful shutdown sequence.
- [ ] Task 6.3: Write tests simulating `SQLITE_BUSY` to verify the backoff logic and stale data annotation.
- [ ] Task 6.4: Run all verification gates (`cargo check --all-features`, `cargo build --no-default-features`, `cargo clippy`, `cargo test`) to guarantee strict compliance with Engineering.md principles.

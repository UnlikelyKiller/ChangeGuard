## Plan: Track 27 - LSP-Lite ChangeGuard Daemon (Phase 21)

### Phase 1: Dependency Management and Isolation
- [ ] Task 1.1: Add `tower-lsp-server` (v0.23.0) and `tokio` (v1.x) to `Cargo.toml` as optional dependencies.
- [ ] Task 1.2: Define the `daemon` feature flag in `Cargo.toml` that enables these dependencies.
- [ ] Task 1.3: Update `deny.toml` to permit `tower-lsp-server`, `tokio`, and their transitive graph (e.g., in `[licenses]`).
- [ ] Task 1.4: Update GitHub CI workflows (`.github/workflows/ci.yml`) to include `cargo test --features daemon` and a compilation check for `cargo build --no-default-features`.

### Phase 2: Subcommand and CLI Boundary
- [ ] Task 2.1: Add `Daemon` variant to the `Commands` enum in `src/cli.rs`, guarded by `#[cfg(feature = "daemon")]`.
- [ ] Task 2.2: Create `src/commands/daemon.rs` with `execute_daemon()`. This configures the minimal Tokio runtime (`worker_threads(2)`) and blocks on it.
- [ ] Task 2.3: Create `src/daemon/mod.rs` to expose the daemon internal modules cleanly.

### Phase 3: Lifecycle and State Management
- [ ] Task 3.1: Create `src/daemon/lifecycle.rs` to handle reading/writing/verifying `.changeguard/daemon.pid`.
- [ ] Task 3.2: Implement stale daemon process detection in `lifecycle.rs`.
- [ ] Task 3.3: Create `src/daemon/state.rs` for read-only SQLite wrapper configuration.
- [ ] Task 3.4: Implement exponential backoff retry (100ms, 200ms, 400ms) for `SQLITE_BUSY` in `state.rs`.

### Phase 4: Data Mapping (The Anti-Corruption Layer)
- [ ] Task 4.1: Create `src/output/lsp.rs` explicitly for transforming internal logic to `tower_lsp_server::ls_types`.
- [ ] Task 4.2: Implement `Diagnostic` mapping logic for ChangeGuard anomalies and AST issues.
- [ ] Task 4.3: Implement `CodeLens` mapping logic for inline temporal/complexity scores.
- [ ] Task 4.4: Implement `Hover` mapping logic for impact/prediction summaries.

### Phase 5: Language Server Implementation
- [ ] Task 5.1: Create `src/daemon/server.rs` and implement the `LanguageServer` trait.
- [ ] Task 5.2: Create `src/daemon/handlers.rs` for modularly resolving the queries using `src/output/lsp.rs` mappers.
- [ ] Task 5.3: Ensure URI resolution leverages `tower_lsp_server::ls_types::UriExt::to_file_path()` to robustly handle Windows vs Unix discrepancies without crashing.
- [ ] Task 5.4: Implement graceful handling of broken stdin/stdout to trigger a 5-second self-termination if the IDE vanishes.

### Phase 6: Edge Case Testing
- [ ] Task 6.1: Add a unit test suite to `src/output/lsp.rs` ensuring deterministic mapping (stable sorting of diagnostics).
- [ ] Task 6.2: Create `tests/daemon_lifecycle.rs` to simulate PID clashes and verify stale process overrides.
- [ ] Task 6.3: Create tests simulating `SQLITE_BUSY` locking to prove the exponential backoff mechanism works without halting.
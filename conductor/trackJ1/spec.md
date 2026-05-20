# Track J1: INFO→DEBUG Log Migration for Storage Init and Enrichment Providers

## Status
In Progress

## Milestone
J: Developer Experience Hardening

## Problem
Two categories of `tracing::info!` calls currently leak to the terminal on virtually every command, even trivial ones like `ledger status --compact`:

1. **Storage init messages** (2–4 lines per command):
   ```
   INFO changeguard::state::storage_cozo: CozoStorage selecting engine 'sled' for path "..."
   INFO changeguard::state::storage_cozo: Initialized CozoDB storage at "..."
   INFO changeguard::state::storage: Initialized storage at "..."
   ```

2. **Impact enrichment provider messages** (14+ lines per `scan --impact` / `impact`):
   ```
   INFO changeguard::impact::orchestrator: Starting impact orchestration...
   INFO changeguard::impact::orchestrator: Running enrichment provider: Federated Intelligence Enrichment Provider
   INFO changeguard::impact::enrichment::api: Skipping API enrichment: api_routes table is empty or missing.
   ... (11 more)
   ```

These messages are internal lifecycle details, not user-actionable output. They obscure the actual command output and make every command feel noisy.

## Root Cause
The calls use `info!` (or `tracing::info!`) which is included in the default `EnvFilter` (`info,...`). Because `--verbose` flag is the only way to go higher, these lines have nowhere to go that is less than full debug.

## Fix Strategy
**Change at source** — the correct fix is to change `info!` to `debug!` at the call sites, not to add more targets to the filter string. This follows the principle that internal infrastructure lifecycle messages belong at DEBUG.

The `src/main.rs` default filter (`info,graph_builder=warn,tantivy=warn,sled=warn`) was introduced by Track I1-3. It should not need to grow indefinitely to suppress internal chatter.

## Scope of Changes

### 1. Storage init messages → DEBUG
- `src/state/storage_cozo.rs`: two `info!` calls for "CozoStorage selecting engine" and "Initialized CozoDB storage"
- `src/state/storage.rs`: one `info!` call for "Initialized storage"

### 2. Impact orchestrator lifecycle → DEBUG
- `src/impact/orchestrator.rs`: `info!("Starting impact orchestration...")` and all `info!("Running enrichment provider: ...")` calls
- Each individual enrichment provider skip message (e.g. "Skipping API enrichment: api_routes table is empty") in `src/impact/enrichment/*.rs`

### 3. Stream indexer worker logs → DEBUG
- `src/search/stream_indexer.rs`: `info!("Worker {}: Indexing file: {}", ...)` — noisy during `search --index`

### 4. Retain as INFO
- The `info!("Performing semantic search for: ...")` in `src/commands/search.rs` should be `debug!` too (user-facing output handles the feedback).
- Any message that is genuinely user-actionable (degraded mode warnings, first-time init notices) **must stay at `info!`** or be upgraded to a user-visible `println!`.

## Success Criteria
- `changeguard ledger status --compact` produces zero INFO lines to stderr.
- `changeguard scan` produces zero INFO lines to stderr.
- `changeguard hotspots` produces zero INFO lines to stderr.
- `changeguard scan --impact` produces zero INFO lines to stderr (the enrichment provider loop is internal).
- `changeguard scan --impact --verbose` still shows all INFO/DEBUG messages.
- `RUST_LOG=debug changeguard ledger status` shows all moved-to-debug messages.
- All existing tests pass.

## Files Changed
- `src/state/storage_cozo.rs`
- `src/state/storage.rs`
- `src/impact/orchestrator.rs`
- `src/impact/enrichment/api.rs` (and other enrichment files with skip-messages)
- `src/search/stream_indexer.rs`
- `src/commands/search.rs`

## Edge Cases
- **First-time init**: If the DB does not exist and is being created for the first time, keep the message at `info!` so users understand why startup is slower. Add a `[init]` tag. (e.g., `info!("[init] Creating new CozoDB storage at ...")`)
- **Error paths**: Any `info!` that actually signals a degraded or recovered state (e.g., "falling back to SQLite path") should stay at `info!` or be elevated to `warn!`.
- **Test assertions**: Some tests may check log output. Update them to use `RUST_LOG=debug` or mock the subscriber at DEBUG level.

## Definition of Done
- [ ] Zero INFO lines on stderr for: `ledger status --compact`, `scan`, `hotspots`, `scan --impact` when not using `--verbose`.
- [ ] `--verbose` restores all moved messages to visible output.
- [ ] `RUST_LOG` override still works as before.
- [ ] First-time DB creation still emits a visible `[init]` message at INFO.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.
- [ ] No regressions in existing test suite.

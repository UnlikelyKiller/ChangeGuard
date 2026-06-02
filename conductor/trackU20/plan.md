# Track U20 Plan: Always-Visible Semantic Index Lifecycle Logging

- [ ] Task U20.1: Write the failing test `lifecycle_log_fires_on_empty_index` capturing tracing events.
- [ ] Task U20.2: Write the failing test `no_println_on_up_to_date_path` asserting stdout/stderr separation.
- [ ] Task U20.3: Move `info!("Semantic indexing threads: ...")` to before the early-exit at `src/commands/index.rs:612`.
- [ ] Task U20.4: Add `info!("Semantic indexing started: incremental={incremental}, cli_concurrency={:?}", ...)` at the top of the function.
- [ ] Task U20.5: Switch `println!("Semantic index is up to date. No files changed.");` to `info!("Semantic index is up to date: no files changed since last index");`.
- [ ] Task U20.6: Add `info!("Semantic indexing will process {} files", files_to_process.len());` between the early-exit and Phase 2.
- [ ] Task U20.7: Run CI gate.
- [ ] Task U20.8: Manual: on a clean repo, `RUST_LOG=info changeguard index --semantic --incremental` shows the lifecycle in order.
- [ ] Task U20.9: Manual: `2>/dev/null changeguard index --semantic --incremental` produces no "up to date" on stdout.
- [ ] Task U20.10: Ledger provenance + commit + push.

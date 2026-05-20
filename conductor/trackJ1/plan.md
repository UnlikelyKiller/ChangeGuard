# Track J1 Plan: INFO→DEBUG Log Migration

## Steps

### Red Phase (failing tests)
1. [ ] Add integration test in `src/state/storage_cozo.rs` that asserts no `INFO` lines appear on stderr when initializing an existing DB (test against a subscriber capturing at DEBUG level)
2. [ ] Add integration test in `src/impact/orchestrator.rs` that asserts no `INFO` lines appear when running a full enrichment pass with an empty corpus (all providers skip)
3. [ ] Run CI gate — tests expected to fail

### Green Phase (implementation)
4. [ ] `src/state/storage_cozo.rs`: Change "CozoStorage selecting engine" and "Initialized CozoDB storage" from `info!` to `debug!`; add `info!("[init] Creating new CozoDB storage at ...")` guard on first-create path
5. [ ] `src/state/storage.rs`: Change "Initialized storage at" from `info!` to `debug!`
6. [ ] `src/impact/orchestrator.rs`: Change "Starting impact orchestration..." and all "Running enrichment provider: ..." calls from `info!` to `debug!`
7. [ ] Each enrichment provider skip message in `src/impact/enrichment/*.rs` (api.rs, kg_provider.rs, coupling.rs, etc.): Change from `info!` to `debug!`
8. [ ] `src/search/stream_indexer.rs`: Change "Worker {}: Indexing file: {}" from `info!` to `debug!`
9. [ ] `src/commands/search.rs`: Change "Performing semantic search for: ..." from `info!` to `debug!`
10. [ ] Run CI gate — all tests expected to pass

### Verification
11. [ ] Smoke-test: `changeguard ledger status --compact 2>&1 | Select-String "INFO"` → zero matches
12. [ ] Smoke-test: `changeguard scan 2>&1 | Select-String "INFO"` → zero matches
13. [ ] Smoke-test: `RUST_LOG=debug changeguard ledger status 2>&1 | Select-String "debug"` → shows moved messages
14. [ ] `changeguard verify` passes

### Finalization
15. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
16. [ ] `changeguard ledger commit` with summary and reason

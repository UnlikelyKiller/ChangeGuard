# Track J5 Plan: KG Enrichment Progress Indicator and Configurable Timeout

## Steps

### Red Phase (failing tests)
1. [ ] Add test in `src/ui/spinner.rs`: `Spinner::start("msg")` and immediate `stop()` do not panic; spinner thread joins cleanly
2. [ ] Add test in `src/impact/enrichment/kg_provider.rs`: when `kg_timeout_secs = 0` is set and KG is unavailable, `enrich_kg()` returns a `ProviderResult` with degraded status rather than an error
3. [ ] Add test: with `kg_timeout_secs = 1` and a mock that sleeps 5s, result is degraded with "timed out" message
4. [ ] Run CI gate — new tests expected to fail

### Green Phase — spinner
5. [ ] Create `src/ui/mod.rs` exporting `spinner` module
6. [ ] Create `src/ui/spinner.rs` with `SpinnerHandle` struct holding `Arc<AtomicBool>` stop flag + `JoinHandle`
7. [ ] `Spinner::start(msg)`: spawn thread that loops printing `\r[|/-\] {msg} {elapsed}s` at 100ms intervals until stop flag set; detect `CI` env var or `TERM=dumb` and emit single `eprintln!` instead
8. [ ] `SpinnerHandle::stop()`: set stop flag, join thread, print `\r` + spaces to clear line (skip on CI)

### Green Phase — KG timeout
9. [ ] Add `kg_timeout_secs: u64` to `KgConfig` or `ImpactConfig` with `serde(default)` returning `60`
10. [ ] In `kg_provider.rs` `enrich()`: move Datalog query into `thread::Builder::new().spawn(...)` 
11. [ ] Use `receiver.recv_timeout(Duration::from_secs(if timeout == 0 { u64::MAX } else { timeout }))` to wait
12. [ ] On `RecvTimeoutError`: emit `warn!`, return `ProviderResult::degraded(format!("KG timed out after {timeout}s"))`
13. [ ] On thread panic (`JoinHandle::join()` returns `Err`): emit `warn!`, return degraded

### Green Phase — orchestrator spinner integration
14. [ ] In `src/impact/orchestrator.rs`: before calling KG enrichment provider, start spinner with `Spinner::start("KG analysis")`; stop after provider returns
15. [ ] Wire `src/ui` module into `src/lib.rs` or `src/main.rs`
16. [ ] Add `kg_timeout_secs = 60` to `.changeguard/config.toml` under appropriate section
17. [ ] Run `cargo build` — fix any type/import errors
18. [ ] Run CI gate — all tests expected to pass

### Verification
19. [ ] `cargo install --path .` to rebuild binary
20. [ ] `changeguard scan --impact` → spinner visible during KG phase (~28s reduced or timed out)
21. [ ] Set `kg_timeout_secs = 5` in config → impact report renders with `[DEGRADED]` KG note after 5s
22. [ ] `CI=true changeguard scan --impact` → no spinner escape codes in output
23. [ ] `changeguard verify` passes

### Finalization
24. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
25. [ ] `changeguard ledger commit` with summary and reason

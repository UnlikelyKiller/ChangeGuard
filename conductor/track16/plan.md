## Plan: Track 16 — Relationship Extraction, Watch Hardening, Verify Results

**Prerequisite**: Track 17 (DB schema expansion) should be merged first for SQLite persistence of verification results and watch batches.

### Phase 1: Import/Export and Runtime Usage Extraction
- [ ] Task 16.1: Add `imports: Option<ImportExport>` and `runtime_usage: Option<RuntimeUsage>` fields to `ChangedFile` in `src/impact/packet.rs`. Update `Serialize`/`Deserialize`. Both default to `None`.
- [ ] Task 16.2: Create `src/index/references.rs`. Define `ImportExport`. Implement extraction for Rust `use` (top-level only, no grouped use), TypeScript `import`/`export`, Python `import`/`from...import` using tree-sitter queries.
- [ ] Task 16.3: Create `src/index/runtime_usage.rs`. Define `RuntimeUsage`. Implement env var detection using regex (not tree-sitter): `std::env::var`, `process.env`, `os.environ`, `os.getenv`. Detect `dotenv`/`config.from_env` as config keys.
- [ ] Task 16.4: Register modules in `src/index/mod.rs`. Wire extraction into `commands/impact.rs` `map_snapshot_to_packet` loop alongside symbol extraction. Wrap in `if let Ok(...) / else None` for graceful degradation.
- [ ] Task 16.5: Write unit tests for each language's import/export extraction. Write unit tests for runtime usage detection. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 2: Watch Command Hardening
- [ ] Task 16.6: Add `ctrlc` crate to `Cargo.toml`. Implement graceful shutdown in `commands/watch.rs`: set up `AtomicBool` running flag, install ctrlc handler that sets flag to false, replace `loop { sleep }` with `while running.load(SeqCst) { sleep }`. Print "Watch mode stopped." on exit.
- [ ] Task 16.7: Wire `WatchBatch::save()` into the watch command callback. Pass `Layout` to the callback closure. Persist each batch to `.changeguard/state/current-batch.json` (overwrite semantics). If Track 17 DB is available, also persist to `batches` table.
- [ ] Task 16.8: Create `src/watch/normalize.rs`. Implement `normalize_event_path(path: &Path, root: &Path) -> PathBuf` with Windows lowercase (`#[cfg]`) + backslash-to-forward-slash conversion + root stripping.
- [ ] Task 16.9: Integrate `normalize_event_path` into `watch/debounce.rs` event processing. Register module in `src/watch/mod.rs`.
- [ ] Task 16.10: **Fix WatchConfig integration**: Pass `WatchConfig` to watcher initialization. Merge `WatchConfig.ignore_patterns` with built-in `EventFilter` hardcoded patterns into a single `GlobSet`. Build the combined set once at watcher creation time.
- [ ] Task 16.11: Write unit tests for path normalization. Write integration test for WatchConfig ignore_patterns filtering. Verify watch mode compiles and runs. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Verification Result Persistence
- [ ] Task 16.12: Create `src/verify/results.rs`. Define `VerificationResult`, `VerificationReport`. Implement `write_verify_report`.
- [ ] Task 16.13: Register in `src/verify/mod.rs`. Wire into `commands/verify.rs` to build and persist report after execution. Set `overall_pass` from exit codes. If Track 17 DB is available, also persist to `verification_runs`/`verification_results` tables.
- [ ] Task 16.14: Write unit tests for report serialization and persistence. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 4: Final Verification
- [ ] Task 16.15: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 16.16: Full suite `cargo test -j 1 -- --test-threads=1`.
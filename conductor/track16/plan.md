## Plan: Track 16 — Relationship Extraction, Watch Hardening, Verify Results

### Phase 1: Import/Export and Runtime Usage Extraction
- [ ] Task 16.1: Add `imports: Option<ImportExport>` and `runtime_usage: Option<RuntimeUsage>` fields to `ChangedFile` in `src/impact/packet.rs`. Update `Serialize`/`Deserialize`.
- [ ] Task 16.2: Create `src/index/references.rs`. Define `ImportExport`. Implement extraction for Rust `use`, TypeScript `import`, Python `import` using tree-sitter queries.
- [ ] Task 16.3: Create `src/index/runtime_usage.rs`. Define `RuntimeUsage`. Implement env var detection (`std::env::var`, `process.env`, `os.environ`), config key detection.
- [ ] Task 16.4: Register modules in `src/index/mod.rs`. Wire extraction into `commands/impact.rs` map_snapshot_to_packet alongside symbol extraction.
- [ ] Task 16.5: Write unit tests for each language's import/export extraction. Write unit tests for runtime usage detection. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 2: Watch Command Hardening
- [ ] Task 16.6: Add `ctrlc` crate to `Cargo.toml`. Implement graceful shutdown in `commands/watch.rs`: replace `loop { sleep }` with channel-based signal handling.
- [ ] Task 16.7: Wire `WatchBatch::save()` into the watch command callback. Persist each batch to `.changeguard/state/current-batch.json`.
- [ ] Task 16.8: Create `src/watch/normalize.rs`. Implement `normalize_event_path(path: &Path, root: &Path) -> PathBuf` with Windows lowercase + backslash conversion + root stripping.
- [ ] Task 16.9: Integrate `normalize_event_path` into `watch/debounce.rs` event processing. Register module in `src/watch/mod.rs`.
- [ ] Task 16.10: Write unit tests for path normalization. Verify watch mode compiles and runs. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Verification Result Persistence
- [ ] Task 16.11: Create `src/verify/results.rs`. Define `VerificationResult`, `VerificationReport`. Implement `write_verify_report`.
- [ ] Task 16.12: Register in `src/verify/mod.rs`. Wire into `commands/verify.rs` to build and persist report after execution.
- [ ] Task 16.13: Write unit tests for report serialization and persistence. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 4: Final Verification
- [ ] Task 16.14: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 16.15: Full suite `cargo test -j 1 -- --test-threads=1`.
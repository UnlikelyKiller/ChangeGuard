## Plan: Track 17 — Engineering Quality

### Phase 1: Remove Unused Dependencies
- [ ] Task 17.1: Audit all `Cargo.toml` dependencies against actual `src/` imports. Remove `ignore`, `blake3`, `once_cell`, `clap_complete`, `clap_mangen` if unused after prior tracks. Keep `regex` if Track 14 added usage.
- [ ] Task 17.2: `cargo build` to confirm no breakage. `cargo clippy` to check for unused import warnings.

### Phase 2: Cross-Platform Verification Tests
- [ ] Task 17.3: Rewrite `tests/cli_verify.rs` to use cross-platform commands. Replace `powershell -Command` with `echo`, `exit`, and platform-conditional sleep.
- [ ] Task 17.4: Add `#[cfg(target_os = "windows")]` and `#[cfg(not(target_os = "windows"))]` attributes where platform-specific behavior is unavoidable.
- [ ] Task 17.5: Verify tests pass on Windows. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Fix Docs Casing
- [ ] Task 17.6: `git mv Docs docs` to rename directory. Update any source references.
- [ ] Task 17.7: Verify no broken paths.

### Phase 4: Expand DB Schema
- [ ] Task 17.8: Add migration for `batches`, `changed_files`, `verification_runs`, `verification_results` tables in `src/state/migrations.rs`.
- [ ] Task 17.9: Add `save_batch`, `save_verification_run` methods to `StorageManager` if needed by watch/verify modules.
- [ ] Task 17.10: Write migration tests using in-memory SQLite. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 5: Config Validation
- [ ] Task 17.11: Implement `validate_config` in `src/config/validate.rs` with field constraints: `debounce_ms > 0`, `timeout_secs > 0`, etc.
- [ ] Task 17.12: Write tests with invalid config values. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 6: Final Verification
- [ ] Task 17.13: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 17.14: Full suite `cargo test -j 1 -- --test-threads=1`.
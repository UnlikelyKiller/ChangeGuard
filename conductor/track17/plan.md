## Plan: Track 17 — Engineering Quality

**Note**: Implement BEFORE Track 16 so the DB schema is ready for verify results and batch persistence.

### Phase 1: Remove Unused Dependencies
- [ ] Task 17.1: Audit all `Cargo.toml` dependencies against actual `src/` imports. Remove `ignore`, `bstr`, `blake3`, `once_cell`, `clap_complete`, `clap_mangen`. Keep `regex` if Track 14 added usage.
- [ ] Task 17.2: `cargo build` after each removal to confirm no breakage. `cargo clippy --all-targets --all-features` to check for unused import warnings. Resolve any warnings.

### Phase 2: Cross-Platform Verification Tests
- [ ] Task 17.3: Rewrite `tests/cli_verify.rs` to use cross-platform commands. Replace `powershell -Command` with `echo hello` / `exit 1`. Use `#[cfg(target_os = "windows")]` and `#[cfg(not(target_os = "windows"))]` for timeout tests (ping vs sleep).
- [ ] Task 17.4: Fix `src/commands/verify.rs` to not wrap `cargo` via `powershell -Command` on Windows. Use direct command execution (cmd.exe can find PATH executables). Fix `src/gemini/mod.rs` similarly — remove `powershell -Command gemini` wrapper, just use `Command::new("gemini")` directly.
- [ ] Task 17.5: Verify tests pass on Windows. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Fix Docs Casing
- [ ] Task 17.6: `git mv Docs docs_temp && git mv docs_temp docs` (two-step for case-insensitive filesystems). Update any source references (import paths, doc links).
- [ ] Task 17.7: Verify no broken paths with `cargo test -j 1 -- --test-threads=1`.

### Phase 4: Expand DB Schema
- [ ] Task 17.8: Add migration for `batches`, `changed_files`, `verification_runs`, `verification_results` tables in `src/state/migrations.rs`. All additive — no changes to existing `snapshots` table.
- [ ] Task 17.9: Add `Migrations::validate()` test as recommended by rusqlite_migration docs. Add migration integration tests using in-memory SQLite: open DB, run migrations, verify all tables exist, insert + query each table.
- [ ] Task 17.10: Add `save_batch`, `save_verification_run`, `save_verification_result`, `save_changed_files` methods to `StorageManager`. Add `get_latest_verification_run()` for query.
- [ ] Task 17.11: Write unit tests for each new StorageManager method. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 5: Config Validation
- [ ] Task 17.12: Implement `validate_config` in `src/config/validate.rs` with field constraints: `debounce_ms > 0`, `timeout_secs > 0`, `model` non-empty if present, `ignore_patterns` valid globs (compile each with globset). Return `ConfigError::ValidationFailed` with field path and constraint.
- [ ] Task 17.13: Wire `validate_config` into `config/load.rs` — call after parsing TOML, before returning config. If validation fails, return the error (do NOT silently fall back).
- [ ] Task 17.14: Write tests with invalid config values (zero debounce, negative timeout, empty model, invalid glob pattern). Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 6: Final Verification
- [ ] Task 17.15: `cargo clippy --all-targets --all-features` and `cargo fmt --check`.
- [ ] Task 17.16: Full suite `cargo test -j 1 -- --test-threads=1`.
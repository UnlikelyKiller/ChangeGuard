# Specification: Track 17 — Engineering Quality

## Overview
Address audit items 15, 17, 18, 19, 20: remove unused dependencies, make tests cross-platform, fix docs casing, expand DB schema, implement config validation.

**Execution order note**: This track should ideally be implemented BEFORE Track 16 because Track 16's verify results persistence and watch batch persistence benefit from the expanded DB schema. If Tracks are implemented sequentially, do Track 17 before Track 16.

## 1. Remove Unused Dependencies
**Priority: MEDIUM** — 6 crates in Cargo.toml are never imported in `src/`.

### Remove
- `ignore` — not used (watch filters use globset)
- `bstr` — not used (gix handles byte strings internally)
- `blake3` — not used (no hashing implemented)
- `once_cell` — not used (Rust 2024 edition has `std::sync::LazyLock`)
- `clap_complete` — not used (no shell completion generation wired)
- `clap_mangen` — not used (no man page generation wired)

### Keep
- `regex` — Track 14 adds usage for secret redaction
- All other dependencies are confirmed used in `src/`

### Decision Rule
- If a dependency is used by a prior track in this batch, keep it
- If truly unused after all prior tracks are merged, remove it
- Run `cargo build` after each removal to confirm

## 2. Cross-Platform Verification Tests
**Priority: MEDIUM**

### `tests/cli_verify.rs`
- Replace `powershell -Command` commands with cross-platform equivalents:
  - Pass tests: Use `echo hello` (works on both bash and cmd.exe on Windows via `cmd /C echo hello`)
  - Fail tests: Use `exit 1` (cross-platform)
  - Timeout tests: Use `sleep 10` (on Unix) or `ping -n 10 127.0.0.1 >nul` (on Windows) — wrap with `#[cfg]` attributes
- Add `#[cfg(target_os = "windows")]` and `#[cfg(not(target_os = "windows"))]` attributes where platform-specific behavior is unavoidable

### `src/commands/verify.rs`
- Fix the Windows-specific `powershell -Command gemini` pattern. Use `which::which("gemini")` or just try the command directly (cmd.exe can find executables in PATH without powershell wrapping). The `powershell -Command` wrapper adds startup latency (~300ms) and is unnecessary for just launching an executable.

## 3. Fix `Docs/` Casing
**Priority: MEDIUM** — Plan specifies `docs/` (lowercase).

- Rename `Docs/` directory to `docs/`
- **Windows caveat**: `git mv Docs docs` may fail on case-insensitive filesystems. Use two-step rename:
  ```bash
  git mv Docs docs_temp
  git mv docs_temp docs
  ```
- Update any references in source code or documentation
- Update `.gitignore` if needed
- Verify no broken paths with `cargo test`

## 4. Expand DB Schema
**Priority: MEDIUM** — Currently only `snapshots` table; plan specifies richer schema.

### New Tables
- `batches`: `id INTEGER PRIMARY KEY`, `timestamp TEXT NOT NULL`, `event_count INTEGER NOT NULL`, `batch_json TEXT NOT NULL`
- `changed_files`: `id INTEGER PRIMARY KEY`, `snapshot_id INTEGER REFERENCES snapshots(id)`, `path TEXT NOT NULL`, `status TEXT NOT NULL`, `is_staged BOOLEAN NOT NULL`
- `verification_runs`: `id INTEGER PRIMARY KEY`, `timestamp TEXT NOT NULL`, `plan_json TEXT`, `overall_pass BOOLEAN NOT NULL`
- `verification_results`: `id INTEGER PRIMARY KEY`, `run_id INTEGER REFERENCES verification_runs(id)`, `command TEXT NOT NULL`, `exit_code INTEGER NOT NULL`, `duration_ms INTEGER NOT NULL`, `truncated BOOLEAN NOT NULL`

### Migration
- Add migration in `src/state/migrations.rs` using `rusqlite_migration` `M::up()` syntax
- Existing `snapshots` table preserved (no breaking changes)
- New tables are additive only
- Use `Migrations::validate()` test as recommended by rusqlite_migration docs

### Storage API
- Add methods to `StorageManager` for new tables:
  - `save_batch(timestamp: &str, event_count: u32, batch_json: &str) -> Result<i64>`
  - `save_verification_run(timestamp: &str, plan_json: Option<&str>, overall_pass: bool) -> Result<i64>`
  - `save_verification_result(run_id: i64, command: &str, exit_code: i32, duration_ms: u64, truncated: bool) -> Result<()>`
  - `save_changed_files(snapshot_id: i64, files: &[ChangedFileRecord]) -> Result<()>`

## 5. Implement Config Validation
**Priority: MEDIUM** — `config/validate.rs` is currently a no-op.

### `src/config/validate.rs`
- Validate `core.strict` is a boolean (serde handles this, but add explicit check for future-proofing)
- Validate `watch.debounce_ms` is > 0 if present
- Validate `watch.ignore_patterns` contains valid glob patterns (try compiling each with `globset`)
- Validate `gemini.timeout_secs` is > 0 if present
- Validate `gemini.model` is a non-empty string if present (not just whitespace)
- Return `ConfigError::ValidationFailed` with field path and constraint description on failure

## Verification
- `cargo build` confirms no unused dependency warnings
- `cargo clippy` confirms no unused import warnings
- Verification tests pass on both Windows and Linux/WSL
- Migration tests for new schema (using in-memory SQLite + `Migrations::validate()`)
- Config validation tests with invalid values (negative debounce_ms, empty model, invalid glob)
- `cargo test -j 1 -- --test-threads=1`
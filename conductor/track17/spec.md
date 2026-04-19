# Specification: Track 17 — Engineering Quality

## Overview
Address audit items 15, 17, 18, 19, 20: remove unused dependencies, make tests cross-platform, fix docs casing, expand DB schema, implement config validation.

## 1. Remove Unused Dependencies
**Priority: MEDIUM** — 6 crates in Cargo.toml are never imported.

### Remove
- `ignore` — not used (watch filters use globset)
- `blake3` — not used (no hashing implemented)
- `regex` — only becoming used in Track 14; if Track 14 is done first, keep it; otherwise remove
- `once_cell` — not used
- `clap_complete` — not used (no shell completion generation wired)
- `clap_mangen` — not used (no man page generation wired)

### Decision Rule
- If a dependency is used by a prior track in this batch, keep it
- If truly unused after all prior tracks are merged, remove it

## 2. Cross-Platform Verification Tests
**Priority: MEDIUM**

### `tests/cli_verify.rs`
- Replace `powershell -Command` commands with cross-platform equivalents
- Use `echo hello` (works on both bash and PowerShell) for pass tests
- Use `exit 1` for fail tests
- Use `sleep` with appropriate platform handling (or `ping` on Windows) for timeout tests
- Conditionally skip tests that require platform-specific behavior

## 3. Fix `Docs/` Casing
**Priority: MEDIUM** — Plan specifies `docs/` (lowercase).

- Rename `Docs/` directory to `docs/`
- Update any references in source code or documentation
- Update `.gitignore` if needed

## 4. Expand DB Schema
**Priority: MEDIUM** — Currently only `snapshots` table; plan specifies richer schema.

### New Tables
- `batches`: id, timestamp, event_count, batch_json
- `changed_files`: snapshot_id, path, status, is_staged
- `verification_runs`: id, timestamp, plan_json, overall_pass
- `verification_results`: run_id, command, exit_code, duration_ms, truncated

### Migration
- Add migration in `src/state/migrations.rs`
- Existing `snapshots` table preserved
- New tables are additive only (no breaking changes)

### Storage API
- Add methods to `StorageManager` for new tables as needed by other modules

## 5. Implement Config Validation
**Priority: MEDIUM** — `config/validate.rs` is currently a no-op.

### `src/config/validate.rs`
- Validate `core.strict` is a boolean (serde handles this, but add explicit check)
- Validate `watch.debounce_ms` is > 0 if present
- Validate `gemini.timeout_secs` is > 0 if present
- Return `ConfigError` with field path and constraint description on failure

## Verification
- `cargo build` confirms no unused dependency warnings
- Verification tests pass on both Windows and Linux/WSL
- Migration tests for new schema
- Config validation tests with invalid values
- `cargo test -j 1 -- --test-threads=1`
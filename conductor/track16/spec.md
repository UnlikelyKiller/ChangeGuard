# Specification: Track 16 — Relationship Extraction, Watch Hardening, Verify Results

## Overview
Address audit items 6, 12, 13, 14: implement Phase 9 relationship/runtime extraction, fix watch command robustness, and persist verification results.

## 1. Import/Export and Runtime Usage Extraction (`src/index/references.rs` and `src/index/runtime_usage.rs`)
**Priority: HIGH** — Phase 9 is entirely missing.

### `src/index/references.rs`
- Extract import/export relationships from each supported language using tree-sitter queries:
  - Rust: `use` statements, `pub` items that are exported
  - TypeScript: `import` statements, `export` declarations
  - Python: `import`/`from ... import` statements
- Define `ImportExport` struct with `imported_from: Vec<String>`, `exported_symbols: Vec<String>`
- Add optional `imports` field to `ChangedFile` in `ImpactPacket`

### `src/index/runtime_usage.rs`
- Detect env var usage patterns: `std::env::var`, `process.env`, `os.environ`
- Detect config key usage: `dotenv`, `config.from_env`, `os.getenv`
- Define `RuntimeUsage` struct with `env_vars: Vec<String>`, `config_keys: Vec<String>`
- Add optional `runtime_usage` field to `ChangedFile` in `ImpactPacket`

### Integration
- Register modules in `src/index/mod.rs`
- Call extraction in `commands/impact.rs` alongside existing symbol extraction
- Update `ImpactPacket` serialization to include new fields
- Graceful degradation: if extraction fails, fields default to `None`

## 2. Watch Command Hardening
**Priority: MEDIUM**

### Graceful Shutdown
- Replace `loop { thread::sleep }` in `commands/watch.rs` with a Ctrl+C handler:
  - Use `ctrlc` crate or `std::sync::mpsc` channel for signal handling
  - On signal, break the loop cleanly and print a "Watch mode stopped" message
  - Ensure the `Watcher` is dropped properly on exit

### Batch Persistence
- After each debounced batch, call `WatchBatch::save()` to persist to `.changeguard/state/current-batch.json`
- Ensure the `Layout` is available in the watch command context

### Path Normalization (`src/watch/normalize.rs`)
- Normalize incoming watcher event paths:
  - Convert to lowercase on Windows for consistent matching
  - Convert backslashes to forward slashes
  - Strip the repo root prefix for repo-relative paths

## 3. Verification Result Persistence (`src/verify/results.rs`)
**Priority: MEDIUM**

### `src/verify/results.rs`
- Define `VerificationResult` struct: command, exit_code, duration, stdout_summary, stderr_summary, truncated, timestamp
- Define `VerificationReport` struct: plan used, results array, overall pass/fail, timestamp
- Implement `pub fn write_verify_report(layout: &Layout, report: &VerificationReport) -> Result<()>` writing to `.changeguard/reports/latest-verify.json`
- Register module in `src/verify/mod.rs`

### Integration
- After verification execution in `commands/verify.rs`, build a `VerificationReport` and persist it
- When using a `VerificationPlan`, execute each step and collect results into the report

## Verification
- Unit tests for import/export extraction per language
- Unit tests for runtime usage detection
- Unit tests for path normalization on Windows-style paths
- Unit tests for verification report serialization and persistence
- Integration test for watch mode with Ctrl+C simulation (if feasible)
- `cargo test -j 1 -- --test-threads=1`
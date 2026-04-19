# Specification: Track 16 — Relationship Extraction, Watch Hardening, Verify Results

## Overview
Address audit items 6, 12, 13, 14: implement Phase 9 relationship/runtime extraction, fix watch command robustness, and persist verification results.

**Dependency note**: Track 17 (DB schema expansion) should be completed BEFORE this track. Track 16's verify results persistence and watch batch persistence should write to SQLite tables created by Track 17, not just to JSON files. If Track 17 is not yet merged, Track 16 writes to JSON files as a fallback with a clear TODO for DB migration.

## 1. Import/Export and Runtime Usage Extraction (`src/index/references.rs` and `src/index/runtime_usage.rs`)
**Priority: HIGH** — Phase 9 is entirely missing.

### `src/index/references.rs`
- Extract import/export relationships from each supported language using tree-sitter queries:
  - **Rust**: `use` statements (capture the path, e.g., `use std::collections::HashMap` → `imported_from: ["std::collections::HashMap"]`). For `pub` items at module level, list as exported. Start simple: only top-level `use` statements, not grouped `use { A, B }` — handle groups in a follow-up.
  - **Typecript**: `import` statements (e.g., `import { Foo } from './bar'` → `imported_from: ["./bar"]`). `export` declarations → `exported_symbols`.
  - **Python**: `import X` and `from X import Y` statements → `imported_from: ["X"]`.
- Define `ImportExport` struct with `imported_from: Vec<String>`, `exported_symbols: Vec<String>`
- Add optional `imports: Option<ImportExport>` field to `ChangedFile` in `ImpactPacket`
- **Keep it simple**: Only extract top-level import/export statements. Don't try to resolve module paths or follow re-exports. The goal is "which modules does this file depend on" not "what is the full dependency graph."

### `src/index/runtime_usage.rs`
- Detect env var usage patterns via simple regex (not tree-sitter — these are string patterns, not AST):
  - Rust: `std::env::var\("([^"]+)"\)`, `env!("[^"]+")`
  - TypeScript: `process\.env\.([A-Z_][A-Z0-9_]*)`, `process\.env\[['"]([^'"]+)['"]\]`
  - Python: `os\.environ\.get\(['"]([^'"]+)['"]\)`, `os\.environ\[['"]([^'"]+)['"]\]`, `os\.getenv\(['"]([^'"]+)['"]\)`
- Detect config key usage (simpler patterns):
  - `dotenv`, `config.from_env`, `os.getenv` — flag the file as "uses env config"
- Define `RuntimeUsage` struct with `env_vars: Vec<String>`, `config_keys: Vec<String>`
- Add optional `runtime_usage: Option<RuntimeUsage>` field to `ChangedFile` in `ImpactPacket`

### Integration
- Register modules in `src/index/mod.rs`
- Call extraction in `commands/impact.rs` alongside existing symbol extraction (inside the map_snapshot_to_packet loop)
- Update `ImpactPacket` serialization to include new fields
- Graceful degradation: if extraction fails, fields default to `None` — never block packet generation for a failed import scan

## 2. Watch Command Hardening
**Priority: MEDIUM**

### Graceful Shutdown
- Replace `loop { thread::sleep }` in `commands/watch.rs` with a Ctrl+C handler:
  - Use `ctrlc` crate (add to Cargo.toml)
  - Set up an `AtomicBool` running flag before starting the watcher
  - On Ctrl+C signal, set the flag to false
  - Main loop: `while running.load(Ordering::SeqCst) { thread::sleep(Duration::from_secs(1)); }`
  - On loop exit, print "Watch mode stopped." and let the `Watcher` drop naturally
  - **Alternative (simpler, no new dep)**: Use `std::sync::atomic::AtomicBool` + `ctrlc` crate, OR just use `std::io::stdin().read_line()` to wait for Enter key to stop. The ctrlc approach is more correct.
- Ensure the `Watcher` is dropped properly on exit (it already has Drop impl via notify_debouncer)

### Batch Persistence
- After each debounced batch, call `WatchBatch::save()` to persist to `.changeguard/state/current-batch.json`
- **Overwrite semantics**: each new batch overwrites the previous file (latest-wins). The timestamp in the batch provides history context.
- Ensure the `Layout` is available in the watch command context (pass it to the callback closure)
- If Track 17 is merged, also persist to the `batches` SQLite table

### Path Normalization (`src/watch/normalize.rs`)
- Normalize incoming watcher event paths:
  - Convert to lowercase on Windows (`#[cfg(target_os = "windows")]`) for consistent matching
  - Convert backslashes to forward slashes (always, both platforms — forward slashes work on Windows)
  - Strip the repo root prefix for repo-relative paths
  - Return `PathBuf` in repo-relative form

### Watch Config Integration
- **Connect `WatchConfig.ignore_patterns`** from config.toml to the `EventFilter`. Currently the filter uses hardcoded patterns. The fix:
  - Pass `WatchConfig` to the watcher initialization
  - Merge `WatchConfig.ignore_patterns` with the built-in hardcoded ignores
  - Build a single `GlobSet` from both sources
  - This addresses a real bug: users who add `ignore_patterns = ["dist/**"]` to config get no effect

## 3. Verification Result Persistence (`src/verify/results.rs`)
**Priority: MEDIUM**

### `src/verify/results.rs`
- Define `VerificationResult` struct: `command: String`, `exit_code: i32`, `duration_ms: u64`, `stdout_summary: String` (first 500 chars), `stderr_summary: String` (first 500 chars), `truncated: bool`, `timestamp: String`
- Define `VerificationReport` struct: `plan: Option<VerificationPlan>`, `results: Vec<VerificationResult>`, `overall_pass: bool`, `timestamp: String`
- Implement `pub fn write_verify_report(layout: &Layout, report: &VerificationReport) -> Result<()>` writing to `.changeguard/reports/latest-verify.json`
- Register module in `src/verify/mod.rs`

### Integration
- After verification execution in `commands/verify.rs`, build a `VerificationReport` and persist it
- When using a `VerificationPlan` (from Track 14), execute each step and collect results into the report
- Set `overall_pass` based on all results having `exit_code == 0`
- If Track 17 is merged, also persist to `verification_runs` and `verification_results` SQLite tables

## Verification
- Unit tests for import/export extraction per language (Rust use, TS import, Python import)
- Unit tests for runtime usage detection (env var patterns per language)
- Unit tests for path normalization on Windows-style paths (`C:\Users\...`, `src\foo\bar.rs`)
- Unit tests for verification report serialization and persistence
- Integration test for watch mode with ctrl+c signal (spawn watch in a thread, send signal, verify clean exit)
- Integration test for WatchConfig.ignore_patterns actually filtering events
- `cargo test -j 1 -- --test-threads=1`
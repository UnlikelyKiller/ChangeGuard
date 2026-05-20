# Track I2-1: Stale Index Warning Banner

**Milestone:** I — Issue Remediation  
**Phase:** 2 — Reliability  
**Issue:** CG-4  
**Status:** In Planning

## Objective

Commands that depend on the Tantivy/CozoDB index (`search`, `ask`, `dead-code`, `hotspots`) silently return degraded results when the index is stale. The index check (`changeguard index --check`) already reports stale file count and last-indexed timestamp. Thread that information into consuming commands as a visible warning banner.

## Requirements

### Config Key
Add to `[index]` section in `DEFAULT_CONFIG` (and `ConfigModel`):
```toml
[index]
stale_threshold_days = 3
```

### Staleness Check Helper
Add a function (e.g., in `src/commands/index.rs` or `src/index/mod.rs`):
```rust
pub fn check_index_staleness(storage: &StorageManager) -> Option<StalenessWarning>
```
Returns `Some(StalenessWarning { stale_files: usize, days_since_indexed: u64 })` if the last-indexed timestamp is older than `stale_threshold_days`, else `None`.

### Warning Banner
On `search`, `ask`, `dead-code`, and `hotspots`, before executing the query, call `check_index_staleness`. If stale, print:
```
⚠  Index is 9 days old with 55 stale files — results may be degraded. Run `changeguard index` to refresh.
```
(Use `eprintln!` or `tracing::warn!` — do not mix into structured JSON output.)

### `--auto-index` Flag
On `search` and `ask` only, add an optional `--auto-index` flag. When set, call `execute_incremental_index()` before the query instead of emitting the banner.

## API Contract

- `StalenessWarning` struct: `{ stale_files: usize, days_since_indexed: u64 }` — internal, no serialization required.
- `--auto-index` appears in `changeguard search --help` and `changeguard ask --help`.
- `--json` output from affected commands must not include the warning banner (banner to stderr only).

## Testing Strategy

- Unit test `staleness_check_fresh`: last-indexed = now; assert returns `None`.
- Unit test `staleness_check_stale`: last-indexed = 10 days ago; assert returns `Some` with correct days count.
- Unit test `staleness_check_threshold_respected`: threshold = 1 day; last-indexed = 2 days ago; assert `Some`.
- No integration test required for the banner output itself.

## Out of Scope

- No change to the actual indexing pipeline.
- `dead-code` and `hotspots` do not get `--auto-index` (those are more expensive operations).

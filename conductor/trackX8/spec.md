# Track X8: `hotspots trend` Shows Human-Readable Timestamps

**Status:** Planned  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

`changeguard hotspots trend` outputs raw RFC3339 timestamps in the tabular display (e.g., `2026-06-08T10:23:45.123456+00:00`). For human scanning, a shorter `YYYY-MM-DD HH:MM` format is clearer and the full precision is noise.

## Problem Statement

In `execute_hotspots_trend`, trend rows are printed as:
```rust
println!("  {} | {} | Score: {:.4}", ts, path, score);
```
`ts` is the raw RFC3339 string stored in SQLite. For a trend report, subsecond precision and the UTC offset are not useful.

## Acceptance Criteria

1. Human-mode trend output formats the timestamp as `YYYY-MM-DD HH:MM UTC`.
2. JSON mode (`--json`) retains the full RFC3339 string (machine contract unchanged).
3. The `chrono` crate (already at `0.4.44`) is used for parsing and formatting — no new dependencies.
4. If a timestamp cannot be parsed (malformed row), it is displayed as-is with no panic.

## Key Files

- `src/commands/hotspots.rs` — `execute_hotspots_trend` (lines 211–228)

## Dependencies

- `chrono = "0.4.44"` (already in Cargo.toml)

## Definition of Done

- `changeguard hotspots trend` shows e.g., `2026-06-08 10:23 UTC` instead of `2026-06-08T10:23:45.123456+00:00`.
- `--json` output is unchanged.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

# Track X8 Plan: `hotspots trend` Human Timestamps

## Phase 1 — Implementation
- [x] 1. In `execute_hotspots_trend` (`src/commands/hotspots.rs`), add a helper:
  ```rust
  fn format_trend_ts(ts: &str) -> String {
      chrono::DateTime::parse_from_rfc3339(ts)
          .map(|dt| dt.with_timezone(&chrono::Utc).format("%Y-%m-%d %H:%M UTC").to_string())
          .unwrap_or_else(|_| ts.to_string())
  }
  ```
- [x] 2. Replace `ts` in the human-mode `println!` with `format_trend_ts(&ts)`.
- [x] 3. Keep `ts` unchanged in the JSON serialization path.

## Phase 2 — Verification
- [x] 4. Run `changeguard hotspots trend`, confirm timestamp format `YYYY-MM-DD HH:MM UTC`.
- [x] 5. Run `changeguard hotspots trend --json`, confirm timestamps are still full RFC3339.
- [x] 6. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 7. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 8. Run `cargo fmt --all -- --check` — clean.
- [x] 9. Update `conductor/conductor.md` status to Completed.

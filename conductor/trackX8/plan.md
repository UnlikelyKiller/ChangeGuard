# Track X8 Plan: `hotspots trend` Human Timestamps

## Phase 1 — Implementation
- [ ] 1. In `execute_hotspots_trend` (`src/commands/hotspots.rs`), add a helper:
  ```rust
  fn format_trend_ts(ts: &str) -> String {
      chrono::DateTime::parse_from_rfc3339(ts)
          .map(|dt| dt.with_timezone(&chrono::Utc).format("%Y-%m-%d %H:%M UTC").to_string())
          .unwrap_or_else(|_| ts.to_string())
  }
  ```
- [ ] 2. Replace `ts` in the human-mode `println!` with `format_trend_ts(&ts)`.
- [ ] 3. Keep `ts` unchanged in the JSON serialization path.

## Phase 2 — Verification
- [ ] 4. Run `changeguard hotspots trend`, confirm timestamp format `YYYY-MM-DD HH:MM UTC`.
- [ ] 5. Run `changeguard hotspots trend --json`, confirm timestamps are still full RFC3339.
- [ ] 6. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 7. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 8. Run `cargo fmt --all -- --check` — clean.
- [ ] 9. Update `conductor/conductor.md` status to Completed.

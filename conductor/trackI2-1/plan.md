# Track I2-1 Plan: Stale Index Warning Banner

## Phase 1 — Red (Failing Tests)

- [ ] Add `StalenessWarning` struct and `check_index_staleness` stub (returns `None` always).
- [ ] Write unit tests:
  - `staleness_check_fresh` — assert `None` when last-indexed = now.
  - `staleness_check_stale` — assert `Some` when last-indexed = 10 days ago and threshold = 3.
  - `staleness_check_threshold_boundary` — last-indexed = 2 days, threshold = 3; assert `None`. Last-indexed = 4 days, threshold = 3; assert `Some`.
- [ ] Commit: `test(index): red — staleness check respects threshold`

## Phase 2 — Green (Implementation)

- [ ] Add `stale_threshold_days = 3` to `DEFAULT_CONFIG` under `[index]`.
- [ ] Add `stale_threshold_days: u64` to the `IndexConfig` or equivalent struct in `src/config/model.rs`.
- [ ] Implement `check_index_staleness`:
  - Read `last_indexed_at` from SQLite (wherever `index --check` reads it from).
  - Parse as RFC3339, compare to `Utc::now()`, return `Some` if delta > threshold.
- [ ] In `src/commands/search.rs`, `ask.rs`, `dead_code.rs`, `hotspots.rs`:
  - Call `check_index_staleness` after storage init.
  - `eprintln!` the banner to stderr if `Some`.
- [ ] In `src/commands/search.rs` and `ask.rs`, add `--auto-index` flag; wire to `execute_incremental_index` when set (skip banner).
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `feat(index): stale index warning banner on search/ask/dead-code/hotspots; --auto-index (CG-4)`

## Verification

- [ ] `changeguard index --check` reports stale files.
- [ ] `changeguard search "ledger"` shows the warning banner (if index is stale).
- [ ] `changeguard search "ledger" --auto-index` triggers re-index instead of banner.
- [ ] `changeguard search "ledger" --json` — confirm banner does not appear in stdout JSON.

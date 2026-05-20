# Track I3-1 Plan: Audit Command Enrichment

## Phase 1 — Red (Failing Tests)

- [ ] Add stub helper functions in `src/commands/audit.rs`:
  - `fetch_drift_summary(conn) -> usize` — returns 0.
  - `fetch_commit_velocity(root) -> CommitVelocity` — returns zeroed struct.
  - `fetch_top_churned_files(root, n) -> Vec<(String, usize)>` — returns empty vec.
  - `fetch_ci_trend(conn) -> CiTrend` — returns empty struct.
  - `fetch_adr_staleness(conn, threshold_days) -> Vec<AdrEntry>` — returns empty vec.
- [ ] Write unit tests for each stub against known inputs (temp SQLite + temp git repo via `git2` or shell).
- [ ] Commit: `test(audit): red — enrichment helpers return correct shapes`

## Phase 2 — Green (Implementation)

- [ ] Implement `fetch_drift_summary`: `SELECT COUNT(*) FROM ledger_entries WHERE status = 'UNAUDITED'`.
- [ ] Implement `fetch_commit_velocity`: shell out to `git log --oneline --format="%ad" --date=short --since="30 days ago"` and aggregate.
- [ ] Implement `fetch_top_churned_files`: shell out to `git log --name-only --since="30 days ago" --format=""` and count file occurrences. Top 5.
- [ ] Implement `fetch_ci_trend`: `SELECT outcome, command, run_at FROM ci_outcome_history ORDER BY run_at DESC LIMIT 30`. Compute pass rate, last failure, most failed command.
- [ ] Implement `fetch_adr_staleness`: `SELECT entity, created_at FROM ledger_entries WHERE category = 'ARCHITECTURE' AND julianday('now') - julianday(created_at) > 180 ORDER BY created_at ASC`.
- [ ] Wire all helpers into `execute_audit()`. Print each section with a header. Use existing color palette.
- [ ] Add `--json` flag: serialize all sections into a `serde_json::Value` and print via `println!("{}", serde_json::to_string_pretty(&v)?)`.
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `feat(audit): multi-section health report with commit velocity, CI trend, ADR staleness (CG-8)`

## Verification

- [ ] `changeguard audit` prints all 7 sections (some may show "no data" for empty tables).
- [ ] `changeguard audit --json` outputs valid JSON with all section keys.
- [ ] `changeguard audit` completes in < 5 seconds on the ChangeGuard repo.

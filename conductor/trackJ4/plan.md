# Track J4 Plan: Global Audit Multi-Section Completion

## Steps

### Red Phase (failing tests)
1. [ ] Add test in `src/commands/ledger_audit.rs`: call `audit_global()` with a temp dir git repo; assert output contains "Commits (30d):"
2. [ ] Add test: output contains "Top churned files:" header
3. [ ] Add test: output contains "Oldest ADR:" header
4. [ ] Add test: output contains "CI trend:" header
5. [ ] Add test: `src/commands/verify.rs` — after a verify run completes, `verify-history.json` exists and contains at least one record
6. [ ] Run CI gate — new tests expected to fail

### Green Phase — verify-history.json writer
7. [ ] Define `VerifyHistoryRecord { timestamp: u64, passed: bool, duration_secs: u64 }` in `src/commands/verify.rs`
8. [ ] After each verify run, read existing `verify-history.json` (empty vec if missing/corrupt), append new record, cap at 100 entries, write back

### Green Phase — audit_global() sections
9. [ ] Add helper `git_commit_count_30d(repo: &Repository) -> Result<u32>` using `git2` revwalk with time filter
10. [ ] Add helper `git_churn_30d(repo: &Repository) -> Result<Vec<(String, u32)>>` — walk commits, count file appearances, return top 5 sorted descending; cap walk at 1000 commits
11. [ ] Add helper `oldest_adr(state_dir: &Path) -> Option<(String, u64)>` — glob `.changeguard/adrs/*.md`, sort by mtime, return filename + age in days
12. [ ] Add helper `load_verify_history(state_dir: &Path) -> Vec<VerifyHistoryRecord>` — read JSON, return empty vec on any error
13. [ ] Add helper `hotspot_delta_section(state_dir: &Path) -> String` — read `hotspot-baseline.json` if present; else return "(no baseline for delta)"
14. [ ] Wire all helpers into `audit_global()` with graceful `unwrap_or`/`unwrap_or_else` degradation for each section
15. [ ] Run `cargo build` — fix any type/import errors
16. [ ] Run CI gate — all tests expected to pass

### Verification
17. [ ] `cargo install --path .` to rebuild binary
18. [ ] `changeguard ledger audit` → all five sections visible
19. [ ] Delete `.changeguard/adrs/` → "Oldest ADR: none found" — no panic
20. [ ] Remove `verify-history.json` → "no verify history recorded" — no panic
21. [ ] `changeguard verify` → `verify-history.json` created/updated
22. [ ] `changeguard verify` passes

### Finalization
23. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
24. [ ] `changeguard ledger commit` with summary and reason

# Track J4: Global Audit Multi-Section Completion

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard ledger audit` (global mode, no path argument) outputs only:

```
PENDING TRANSACTIONS: None
```

The I3-1 spec defined five additional sections — commit velocity (30d), top churned files, CI trend, oldest ADR, and hotspot delta — but none were implemented. The command gives no engineering intelligence beyond what `ledger status` already shows, making it effectively redundant.

## Fix Strategy
Implement the five sections in `src/commands/ledger_audit.rs` `audit_global()` using data already available from existing ChangeGuard subsystems:

| Section | Data Source |
|---------|-------------|
| Commit velocity (30d) | `git log --since=30.days --oneline` count via `git2` |
| Top churned files (30d) | `git log --since=30.days --name-only` + frequency count |
| Oldest open ADR | Scan `.changeguard/adrs/` for the oldest file by mtime |
| Hotspot delta (30d) | ChangeGuard hotspot scores now vs. 30 days ago (approximated via git blame complexity) |
| CI trend | Read `.changeguard/reports/verify-history.json` if present; else "No CI history recorded" |

Each section degrades gracefully: if the data source is unavailable (git error, missing file), the section prints `  (unavailable: <reason>)` rather than failing.

## Scope of Changes

### 1. `src/commands/ledger_audit.rs` → `audit_global()`
Add five sections after the existing PENDING TRANSACTIONS block:

**Section: Commit Velocity (30 days)**
- Query git log via `git2::Repository` for commits since 30 days ago.
- Display: `  Commits (30d): 42`
- Degrade: `  Commits (30d): unavailable (no git repository)`

**Section: Top Churned Files (30 days)**
- Walk commits from past 30 days; count per-file appearances in diffs.
- Display top 5 files with count: `  src/commands/verify.rs  ×18`
- Degrade: `  Top churned files: unavailable`

**Section: Oldest Open ADR**
- List `.changeguard/adrs/*.md` sorted by mtime ascending; show filename + age in days.
- Display: `  Oldest ADR: 0014-embedding-pipeline.md (47 days old)`
- Degrade: `  Oldest ADR: none found`

**Section: Hotspot Delta**
- Compare current hotspot scores from `changeguard hotspots --json` output vs. a cached baseline stored in `.changeguard/reports/hotspot-baseline.json`.
- If no baseline exists, show current top-5 and note "(no baseline for delta)".
- Display: `  src/commands/verify.rs  score +0.03 (was 0.12, now 0.15)`
- Degrade: `  Hotspot delta: unavailable`

**Section: CI Trend**
- Read `.changeguard/reports/verify-history.json` (array of `{timestamp, passed: bool, duration_secs: u64}`).
- Show last 5 runs and pass rate.
- Degrade: `  CI trend: no verify history recorded`

### 2. `.changeguard/reports/verify-history.json`
- `src/commands/verify.rs` should append a record after each run (pass/fail, duration). If the file does not exist, create it. Cap at 100 records.
- This enables the CI Trend section without external dependencies.

## Success Criteria
- `changeguard ledger audit` (no args) prints all five sections.
- Every section degrades gracefully when its data source is missing.
- Output is structured and scannable (section headers, consistent indentation).
- `changeguard ledger audit --compact` (if flag exists) continues to work.
- `src/commands/verify.rs` appends a record to `verify-history.json` after each run.

## Files Changed
- `src/commands/ledger_audit.rs`
- `src/commands/verify.rs`

## Edge Cases
- **Git repo not initialized**: All git-dependent sections degrade gracefully with a message.
- **`.changeguard/adrs/` does not exist**: Oldest ADR section shows "none found".
- **`verify-history.json` corrupt/unreadable**: CI trend section shows "no verify history recorded" (do not panic).
- **`verify-history.json` > 100 records**: Trim oldest entries on write to keep file small.
- **Hotspot baseline missing**: Show current top-5 with "(no baseline for delta)" note. Do not create the baseline automatically; require an explicit `changeguard hotspots --save-baseline` command (out of scope for J4 — document as future work).
- **Very slow git log on large repos**: Cap git log walk at 1000 commits. Emit `debug!` if cap is hit.

## Definition of Done
- [ ] `changeguard ledger audit` shows: pending transactions, commit velocity (30d), top churned files (30d), oldest ADR, hotspot delta, CI trend.
- [ ] Each section degrades gracefully (no panic, clear "(unavailable: …)" message) when data is missing.
- [ ] `src/commands/verify.rs` appends to `verify-history.json` after each verify run.
- [ ] Git log walk is capped at 1000 commits.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.

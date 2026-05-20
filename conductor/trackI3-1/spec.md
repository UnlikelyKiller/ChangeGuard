# Track I3-1: Audit Command Enrichment

**Milestone:** I — Issue Remediation  
**Phase:** 3 — Feature Depth  
**Issue:** CG-8  
**Status:** In Planning

## Objective

`changeguard audit` currently only prints pending transactions. It has access to rich data across SQLite tables (`ledger_entries`, `ci_outcome_history`, `token_provenance`, etc.) and git history, but surfaces none of it. Transform `audit` into a multi-section project health report.

## Required Sections

All sections are present in the default output. Each section is omittable via a future `--only <section>` flag (not required in this track).

### 1. Pending Transactions
Existing behavior — keep unchanged.

### 2. Unaudited Drift Summary
Count of `UNAUDITED` entries in `ledger_entries`. Actionable hint: "Run `changeguard ledger reconcile` to resolve."

### 3. Commit Velocity (last 30 days)
Use `git log --oneline --since="30 days ago"` via `std::process::Command`. Report: total commits, commits per week (rolling average), busiest day.

### 4. Top Churned Files (last 30 days)
Use `git log --name-only --since="30 days ago"` or read from `temporal_couplings` / `hotspots` tables if populated. Report top 5 files by change frequency.

### 5. CI Gate Trend (last 30 runs)
Query `ci_outcome_history` table. Report: pass rate %, most recent failure, most failed command. If the table is empty, report "No CI history recorded yet."

### 6. ADR Staleness
Query `ledger_entries` for entries with `category = 'ARCHITECTURE'` that have not been updated in > 180 days. Report count and the oldest entry's `entity` and `created_at`.

### 7. Hotspot Delta
Query `hotspots` score for the top file now vs. the score from the previous `audit` run (stored in a new `audit_snapshots` table or read from the report file). If no baseline, report "First audit — no delta available."

### `--json` Flag
When `--json` is passed, emit the entire report as a JSON object with keys matching the section names. Do not print colored output in JSON mode.

## API Contract

```
changeguard audit [--json]
```

New table: `audit_snapshots` (optional, for hotspot delta):
- `id INTEGER PRIMARY KEY`
- `run_at TEXT NOT NULL`
- `top_hotspot_file TEXT`
- `top_hotspot_score REAL`

If adding a new migration is too heavyweight for this track, hotspot delta can read the last `latest-impact.json` report file instead of a DB table.

## Testing Strategy

- Unit tests for each data-extraction helper (commit velocity, CI trend, ADR staleness) using temp SQLite and a mock git repo.
- Integration test: run `changeguard audit --json` on the ChangeGuard repo itself; assert the JSON parses and contains all 7 section keys.

## Out of Scope

- `--only <section>` filtering flag.
- Historical trending (sparklines, graphs).
- Automatic remediation suggestions beyond the existing actionable hints.

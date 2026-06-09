# Track K9: Unified Audit Reporting

## Status
Completed

## Milestone
K: Service Discovery & Storage Hardening

## Problem
The `ledger audit` output is disjointed. Global summaries (velocity, churn) are always shown, while the paginated "RECENT COMMITTED ENTRIES" table feels like an appendage. The `--limit` flag only applies to the entries table, not the holistic report scope.

## Objective
Refactor the audit command into a unified reporting abstraction.

## Success Criteria
- [x] `ledger audit` output uses a consistent "Report" frame.
- [x] `--limit` and `--offset` are applied consistently (e.g., top churned files also respect limit).
- [x] Implementation uses a `ProjectAuditReport` struct to separate data gathering from rendering.
- [x] CI gate passes.

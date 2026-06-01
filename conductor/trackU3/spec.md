# Track U3 Spec: Proactive SQLite PRAGMA Auditing

## Background
SQLite contains PRAGMA statements that return results (e.g. `PRAGMA integrity_check`, `PRAGMA wal_checkpoint`, `PRAGMA index_list`). In `rusqlite`, calling `.execute()` on any query that returns rows throws a runtime `ExecuteReturnedResults` error, which can halt index updates or nightly maintenance sweeps.

## Objective
Audit and refactor all PRAGMA executions across both ChangeGuard and AI-Brains source trees to ensure safety and contract compliance.

## Proposed Design
* Query both codebases for all PRAGMA invocations (e.g. `PRAGMA journal_mode`, `PRAGMA busy_timeout`, `PRAGMA wal_checkpoint`, `PRAGMA integrity_check`).
* Verify whether each pragma returns rows:
  * Non-result statements (like settings `PRAGMA journal_mode = WAL`) can safely use `.execute()` or `.execute_batch()`.
  * Query/status pragmas (like `PRAGMA integrity_check`) must use `.query_row()` or `.query_map()`.
* Correct any misaligned invocations and add regression tests.

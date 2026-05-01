# Specification: Track E0-2 Hotspot Complexity Fallback

## Overview

The `calculate_hotspots` function in `src/impact/hotspots.rs` queries the
`symbols` table for per-file complexity data. When no prior `impact` run has
populated `symbols`, every file gets complexity 0, making the hotspot scoring
meaningless (all scores become 0). This track adds a fallback: when the
`symbols` table returns no data for a file, query the `project_symbols` table
(from Track E1-1) as an alternative source. Because `project_symbols` does not
exist yet, the fallback must degrade gracefully -- if the table does not exist,
return 0 complexity (current behavior) rather than crashing.

## Components

### 1. Complexity Source Abstraction (`src/impact/hotspots.rs`)

Currently `calculate_hotspots` runs a single SQL query against the `symbols`
table:

```sql
SELECT file_path, MAX(IFNULL(cognitive_complexity, 0), IFNULL(cyclomatic_complexity, 0)) as max_comp
FROM symbols
GROUP BY file_path
```

Replace this with a two-tier lookup:

1. **Primary**: Query the `symbols` table as before.
2. **Fallback**: For file paths not found in the primary result, query
   `project_symbols` using:

   ```sql
   SELECT file_path, MAX(IFNULL(cognitive_complexity, 0), IFNULL(cyclomatic_complexity, 0)) as max_comp
   FROM project_symbols
   GROUP BY file_path
   ```

3. **Merge**: Combine primary and fallback results using per-file gap-fill.
   Primary results take precedence for any file that appears in both tables.
   Fallback results fill gaps only for files absent from the primary (`symbols`)
   table. This is **not** an all-or-nothing fallback: if `symbols` has data for
   5 files and `project_symbols` has data for 10 different files, the merged
   result contains all 15 files with `symbols` data winning for any overlap.

### 2. Graceful Degradation (`src/impact/hotspots.rs`)

The `project_symbols` table is introduced by Track E1-1 (Migration M15) and
does not exist yet. The fallback query must handle this:

- If the `project_symbols` table does not exist (SQLite returns an error for
  the query), log a `tracing::debug!` message and continue with primary results
  only. Do not crash.
- If the table exists but is empty, that is fine -- the fallback simply
  contributes no additional data.
- Use `rusqlite` in a way that detects the "no such table" error
  (`SQLITE_ERROR` with message containing "no such table") rather than
  panicking.

### 3. Fallback Function Signature (`src/impact/hotspots.rs`)

Extract the complexity query into a helper function:

```rust
fn query_file_complexities(
    conn: &rusqlite::Connection,
    primary_table: &str,
    fallback_table: Option<&str>,
) -> HashMap<String, i32>
```

This function:
- Queries `primary_table` (always `"symbols"`).
- If `fallback_table` is `Some`, queries that table and merges results.
- Returns the merged `HashMap<String, i32>`.

The `calculate_hotspots` function calls this helper instead of the inline query.

### 4. StorageManager Extension (`src/state/storage.rs`)

Add a utility method `table_exists(conn: &Connection, table_name: &str) -> bool`
that checks SQLite's `sqlite_master` for the table. This avoids attempting a
query that will fail and is cleaner than catching errors.

Alternatively, the fallback query can simply catch the rusqlite error. Either
approach is acceptable; prefer whichever is simpler and already consistent
with the codebase style.

### 5. Unit Tests (`src/impact/hotspots.rs` or `tests/`)

- `test_hotspots_uses_symbols_when_available`: Set up a test DB with `symbols`
  data, verify hotspots return that complexity.
- `test_hotspots_falls_back_to_project_symbols`: Set up a test DB with no
  `symbols` data but `project_symbols` data, verify hotspots return the
  `project_symbols` complexity.
- `test_hotspots_prefers_symbols_over_project_symbols`: Set up both tables with
  different values for the same file, verify `symbols` values take precedence.
- `test_hotspots_graceful_degradation_without_project_symbols`: Set up a test DB
  with `symbols` data but no `project_symbols` table, verify hotspots still
  work with `symbols` data only (no crash).

## Constraints & Guidelines

- **No migration dependency**: This track must compile and work without the
  `project_symbols` table. The `project_symbols` table is defined in Track
  E1-1 (Migration M15), which has not landed yet. The fallback path must handle
  the absent table gracefully.
- **No performance regression**: The fallback query must not make hotspots
  materially slower. A single additional `SELECT ... GROUP BY` on
  `project_symbols` is acceptable because the table is indexed by `file_path`
  (it will be when E1-1 creates it) and the query runs once.
- **Backward compatible**: When `project_symbols` does not exist or is empty,
  behavior is identical to the current implementation.
- **Primary precedence**: Data from `symbols` always wins over
  `project_symbols` for the same file path. The `symbols` table holds
  impact-run-specific data that is more precise for the current change context.
- **TDD**: Write tests first, confirm they fail, then implement.

## Acceptance Criteria

1. `calculate_hotspots` returns non-zero complexity scores when `project_symbols`
   has data, even when `symbols` is empty.
2. `calculate_hotspots` prefers `symbols` data over `project_symbols` data when
   both exist for the same file path.
3. `calculate_hotspots` does not crash when the `project_symbols` table does not
   exist in the database.
4. `calculate_hotspots` does not crash when the `project_symbols` table exists
   but is empty.
5. Existing hotspot tests continue to pass (no regression).
6. A debug-level log message is emitted when `project_symbols` is unavailable,
   not an error or warning.

## Definition of Done

- All acceptance criteria pass
- All unit tests pass
- `cargo fmt --all -- --check` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
- `cargo test` passes with no regressions
- No deviations from this spec without documented justification
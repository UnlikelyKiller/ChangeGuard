# Track X3: `hotspots explain` Shows Correct Complexity and Frequency

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** High

## Objective

`changeguard hotspots explain <file>` reports `Complexity: 0` and `Change Frequency: 0.00` for the #1 ranked hotspot file. Both values come from separate queries that fail to match due to path normalization mismatches between what the user passes, what SQLite stores, and how the hotspot engine filters.

## Problem Statement

Two independent bugs:

1. **Complexity=0**: The SQL query joins `project_symbols` and `project_files` using `pf.file_path = ?1`. The user passes a relative path (e.g., `src/commands/hotspots.rs`) but `project_files.file_path` stores absolute Windows paths (`C:\dev\ChangeGuard\src\commands\hotspots.rs`). No row matches → complexity defaults to 0.

2. **Frequency=0.00**: `calculate_hotspots` is called with `HotspotQuery { dir_filter: Some(entity.clone()), ..Default::default() }`. The `dir_filter` field is used as a *directory prefix* filter (the variable is named `dir_filter`), not an exact file match. A file path like `src/commands/hotspots.rs` doesn't prefix-match anything in the way the filter expects, so no hotspot rows are returned and `hotspots.first()` is `None`.

## Acceptance Criteria

1. `hotspots explain src/commands/hotspots.rs` resolves to the correct absolute path before the SQL complexity query.
2. `hotspots explain` passes the entity path as an *exact file* filter (not dir prefix) to `calculate_hotspots`, returning the matching hotspot.
3. If the file is not found (bad path), a clear error is returned: `"No data found for '{entity}'. Run 'changeguard index' to populate metrics."`.
4. Temporal coupling filter uses `to_string_lossy()` comparison after path normalization, not raw string equality.
5. All existing hotspot tests pass; one new unit test covers path normalization in explain.

## API Contracts

```
changeguard hotspots explain <entity>
```

`<entity>` accepts:
- Relative path from repo root: `src/commands/hotspots.rs`
- Absolute path: `C:\dev\ChangeGuard\src\commands\hotspots.rs`

## Key Files

- `src/commands/hotspots.rs` — `execute_hotspots_explain` (lines 231–283)
- `src/impact/hotspots.rs` — `HotspotQuery.dir_filter` field and filtering logic

## Definition of Done

- `changeguard hotspots explain src/commands/hotspots.rs` shows non-zero complexity (≥ 1 if the file has any indexed symbols) and non-zero frequency (≥ 0.01 if the file appears in git history).
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

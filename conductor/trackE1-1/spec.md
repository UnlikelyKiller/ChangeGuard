# Specification: Track E1-1 Full-Project Symbol Index

## Overview

Implement the foundation track of Phase E1 (Structural Bearings). This track adds a `changeguard index` command that scans all supported source files in the repository, extracts symbols, imports, exports, and complexity metrics, and stores the results in SQLite tables (`project_files` and `project_symbols`). This fixes the "0 complexity on fresh repos" hotspot gap by providing a fallback data source independent of the `impact` command's `symbols` table.

The existing `symbols` table (populated during `impact` runs) remains unchanged. `project_symbols` is a separate, always-current index that `hotspots` can query when `symbols` has no data for a file.

**E1-1 owns the shared Migration M15**, which must include ALL E1 tables plus the F0/F1 tables (`project_files`, `index_metadata`, `project_symbols`, `project_docs`, `project_topology`).

## Components

### 1. Database Migration M15 — Shared Foundation (`src/state/migrations.rs`)

Add a new migration (M15) after the existing M14 (token_provenance). This migration is shared across tracks E1-1 through E1-4 plus the F0/F1 foundation tracks. **E1-1 owns M15** — all tracks coordinate with E1-1 for schema additions.

M15 must include ALL tables from F0, F1, E1-1, E1-2, and E1-3 in a single migration: `project_files` (F0), `index_metadata` (F0), `project_symbols` (E1-1), `project_docs` (E1-2), and `project_topology` (E1-3). The `project_symbols` table also includes the `entrypoint_kind` column required by E1-4.

**`project_files` table (F0):**
```sql
CREATE TABLE IF NOT EXISTS project_files (
    id              INTEGER PRIMARY KEY,
    file_path       TEXT NOT NULL,
    language        TEXT,
    content_hash    TEXT,
    git_blob_oid    TEXT,
    file_size       INTEGER,
    mtime_ns        INTEGER,
    parser_version  TEXT NOT NULL DEFAULT '1',
    parse_status    TEXT NOT NULL DEFAULT 'OK',
    last_indexed_at TEXT NOT NULL,
    UNIQUE(file_path)
);
```

**`index_metadata` table (F0):**
```sql
CREATE TABLE IF NOT EXISTS index_metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

Stored keys: `schema_version`, `index_version`, `tree_sitter_query_version`, `last_git_head`, `last_indexed_at`, `workspace_root`, `path_normalization_version`.

**`project_symbols` table (E1-1, with E1-4 `entrypoint_kind` column):**
```sql
CREATE TABLE IF NOT EXISTS project_symbols (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id               INTEGER NOT NULL REFERENCES project_files(id),
    qualified_name        TEXT NOT NULL,
    symbol_name           TEXT NOT NULL,
    symbol_kind           TEXT NOT NULL,
    visibility            TEXT,
    entrypoint_kind       TEXT NOT NULL DEFAULT 'INTERNAL',
    is_public             INTEGER DEFAULT 0,
    cognitive_complexity  INTEGER,
    cyclomatic_complexity  INTEGER,
    line_start            INTEGER,
    line_end              INTEGER,
    byte_start           INTEGER,
    byte_end             INTEGER,
    signature_hash       TEXT,
    confidence           REAL NOT NULL DEFAULT 1.0,
    evidence             TEXT,
    last_indexed_at      TEXT NOT NULL,
    UNIQUE(file_id, qualified_name, symbol_kind)
);
CREATE INDEX IF NOT EXISTS idx_project_symbols_file
    ON project_symbols(file_id);
CREATE INDEX IF NOT EXISTS idx_project_symbols_qualified
    ON project_symbols(qualified_name);
CREATE INDEX IF NOT EXISTS idx_project_symbols_name
    ON project_symbols(symbol_name);
CREATE INDEX IF NOT EXISTS idx_project_symbols_kind
    ON project_symbols(symbol_kind);
CREATE INDEX IF NOT EXISTS idx_project_symbols_entrypoint
    ON project_symbols(entrypoint_kind);
```

- All columns are additive. No existing tables or columns are modified.
- The `file_id` foreign key references `project_files(id)`. SQLite foreign keys are advisory; no cascading deletes at this stage.
- **No downstream table may rely solely on `file_path + symbol_name` for joins** (constraint #7 from expansion plan). All identity references use integer foreign keys (`file_id`, `symbol_id`) with fallback text fields only for unresolved cases.
- `cognitive_complexity` and `cyclomatic_complexity` are nullable (`INTEGER`, not `INTEGER NOT NULL`) to match the existing Rust `Option<i32>` type.
- `confidence REAL NOT NULL DEFAULT 1.0` and `evidence TEXT` columns are included on the `project_symbols` table per the expansion plan requirement that every extracted fact must carry a confidence score and evidence string.
- The `Symbol` struct in Rust must be extended with `line_start`, `line_end`, `qualified_name`, `byte_start`, `byte_end` fields to populate these columns.

### 2. `changeguard index` Command (`src/commands/index.rs`)

New CLI command that performs full or incremental project indexing.

**CLI interface:**
```
changeguard index [--incremental] [--check]
```

- `--incremental`: Only re-index files whose content has changed since the last `index` run. Uses the three-part invalidation key from F1: compare `content_hash` (BLAKE3), `git_blob_oid`, and `parser_version` from `project_files` against current values. Files with no entry or stale values are re-indexed. Files unchanged since last indexing are skipped. This replaces mtime-only comparison, which is unreliable on Windows.
- `--check`: Show indexing status (number of indexed modules, stale files, unsupported files) without re-indexing. Exits with code 0 if the index is fresh, code 1 if re-indexing is recommended.
- No flags (full index): Clear `project_files` and `project_symbols`, scan all supported source files, populate tables from scratch.

**File discovery:**
- Use `gix` to list all tracked files in the working tree (respecting `.gitignore`).
- Filter by supported language extensions: `.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`.
- Skip binary files by extension list: `.png`, `.jpg`, `.jpeg`, `.gif`, `.ico`, `.woff`, `.woff2`, `.ttf`, `.eot`, `.pdf`, `.zip`, `.tar`, `.gz`, `.exe`, `.dll`, `.so`, `.dylib`, `.wasm`, `.class`, `.jar`, `.pyc`.

**Indexing pipeline (per file):**
1. Read file content from disk (use `std::fs::read_to_string`).
2. Dispatch to the existing `src/index/languages/` extractors via `parse_symbols()`.
3. Compute `avg_complexity` as the mean `cognitive_complexity` of all symbols in the file (0 if no symbols have complexity).
4. Compute `symbol_count` as the total number of symbols extracted.
5. Insert into `project_files` (one row per file) and `project_symbols` (one row per symbol).
6. On parse failure: set `parse_status = 'PARSE_FAILED'` in `project_files`, and continue. Accumulate a list of failed files to report as warnings at the end.

**Incremental indexing:**
- Read all `project_files` rows and `index_metadata` for `last_git_head` and `parser_version`.
- For each tracked file, compare the stored `content_hash` (BLAKE3 of file content) and `parser_version` against current values. If either differs, re-index that file.
- If `last_git_head` in `index_metadata` differs from the current HEAD, mark all files for re-validation (the git tree may have changed).
- If `parser_version` has changed globally (e.g., tree-sitter query update), force full re-index.
- Delete old `project_symbols` rows for re-indexed files before re-inserting.
- Files not in the repository but present in `project_files` (deleted files) are marked with `parse_status = 'DELETED'`, not removed (preserves historical references per F1).

**Large repository handling:**
- Batch inserts every 500 files (commit the SQLite transaction, then start a new one).
- Show progress via `indicatif` progress bar: `Indexing: 123/2000 files...`
- Cap: if more than 10,000 source files are found, warn the user and index the first 10,000. This prevents unbounded memory usage.

### 3. `src/index/project_index.rs` (New Module)

New module that orchestrates the full-project indexing logic, separate from the per-impact-run extraction in `src/index/symbols.rs`.

**Key types:**
```rust
pub struct ProjectIndexer {
    storage: StorageManager,
    repo_path: Utf8PathBuf,
}

pub struct IndexStats {
    pub files_indexed: usize,
    pub symbols_indexed: usize,
    pub parse_failures: usize,
    pub skipped_binary: usize,
    pub skipped_unsupported: usize,
    pub duration_ms: u64,
}
```

**Key methods:**
- `ProjectIndexer::new(storage, repo_path) -> Self`
- `ProjectIndexer::full_index(&self) -> Result<IndexStats>`: Clear existing data, scan all files, index all.
- `ProjectIndexer::incremental_index(&self) -> Result<IndexStats>`: Only re-index changed files.
- `ProjectIndexer::check_status(&self) -> Result<IndexStatus>`: Report index freshness without re-indexing.
- `ProjectIndexer::file_for_path(&self, path: &str) -> Result<Option<ProjectFile>>`: Query a single file by path.
- `ProjectIndexer::symbols_for_file(&self, file_id: i64) -> Result<Vec<ProjectSymbol>>`: Query symbols for a file by `file_id`.

### 4. Hotspot Fallback Integration (`src/impact/hotspots.rs`)

When the `symbols` table returns no complexity data for a file (complexity = 0 or missing), fall back to `project_symbols`:

- In the `calculate_hotspots` function (or the complexity lookup it delegates to), after querying `symbols`, if a file has no row or has complexity 0, query `project_symbols` for that file's symbols and compute complexity from there.
- If `project_symbols` also has no data, the file retains complexity 0 (graceful degradation).
- This is the fix for Gap 5.2 ("Hotspot complexity is 0 without a prior `impact` run").

### 5. Module Registration (`src/index/mod.rs`)

Add `pub mod project_index;` to `src/index/mod.rs`.

### 6. Command Registration (`src/commands/mod.rs`)

Add `pub mod index;` to `src/commands/mod.rs`.

Wire the `index` subcommand into the CLI (`src/cli.rs` or wherever `clap` commands are defined).

## Constraints

- **No breaking changes to existing CLI.** The `index` command is a new subcommand. Existing `impact`, `hotspots`, `verify`, and `ledger` commands must produce identical output when `project_symbols` is empty.
- **Single binary.** All new code is in the `changeguard` crate. No new build dependencies except those already approved (`pulldown-cmark` for E1-2, not this track).
- **Local-first.** No network calls. All indexing is from the local filesystem and git repository.
- **Graceful degradation.** If the `project_files` or `project_symbols` tables do not exist (pre-M15 database), `hotspots` must still work with the legacy `symbols` table. The `index` command should run the M15 migration on first use.
- **Performance targets.** Full index of a 2,000-file repo must complete in under 30 seconds. Incremental index of 5 changed files must complete in under 1 second. `hotspots` must not regress beyond the existing 10-second target for 10,000 commits.
- **Streaming inserts.** Use SQLite transactions batched every 500 files. Do not hold all data in memory before inserting.

## Edge Cases

- **Very large repos (>10,000 source files):** Cap indexing at 10,000 files. Emit a warning listing how many files were skipped. Users can set a config option or use a narrower path filter in a future phase.
- **Parse failures:** A file that cannot be parsed (malformed syntax, encoding errors) gets a `project_files` row with `parse_status = 'PARSE_FAILED'`. The file path is added to a warning list printed at the end. Never crash on a parse failure.
- **Mixed-language repos:** Dispatch per-language using the existing `parse_symbols()` dispatcher. Accumulate results from all languages. Report unsupported file types as `UNSUPPORTED` in the stats summary.
- **Binary files:** Skip by extension list before attempting to read content. Count skipped files in `IndexStats.skipped_binary`.
- **Incremental index after git operations (rebase, merge):** Use `gix` to diff HEAD vs last-indexed commit. If the last-indexed commit is no longer an ancestor of HEAD, fall back to a full index. This handles force-push and rebase scenarios.
- **Concurrent runs:** Use a SQLite WAL lock. If the database is locked by another `index` process, emit an error and exit rather than blocking or corrupting.
- **Empty repository:** If no supported source files exist, `index` succeeds with `modules_indexed = 0`. No warnings (it is valid for a repo to have no source files, e.g., a docs-only repo).
- **Files with no symbols (e.g., config files, empty modules):** Create a `project_files` row with `parse_status = 'OK'` but do not create any `project_symbols` rows.
- **Symbol name collisions across files:** The `project_symbols` table allows the same `symbol_name` in different files. The `idx_project_symbols_symbol_name` index supports lookup by name across all files.

## Acceptance Criteria

1. `changeguard index` populates `project_files` and `project_symbols` for all supported source files (`.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`).
2. `changeguard index --incremental` only re-indexes files whose content_hash, git_blob_oid, or parser_version has changed since the last full or incremental index (F1 invalidation logic, not mtime-only).
3. `changeguard index --check` reports index status without modifying the database.
4. `changeguard hotspots` on a fresh repo (no prior `impact` run) returns meaningful, non-zero complexity scores from the project index.
5. Incremental re-index takes under 1 second when only 5 files changed (measured on a fixture repo).
6. Parse failures are surfaced as warnings in the command output, never as crashes.
7. Full index of a 500-file fixture repo completes in under 10 seconds.
8. The `index` command runs the M15 migration automatically on first use. M15 includes ALL E1 tables plus F0/F1 tables (`project_files`, `index_metadata`, `project_symbols`, `project_docs`, `project_topology`). E1-1 owns M15.
9. `hotspots` command produces identical output when `project_symbols` is empty (graceful degradation).
10. No existing test suite regressions.

## Verification Gate

- **Integration test:** Init a fixture repo, run `changeguard index`, then run `changeguard hotspots`. Verify that non-zero complexity scores are returned and `project_symbols` is non-empty.
- **Unit tests:** CRUD operations for `project_files` and `project_symbols` (insert, query, delete, update).
- **Unit tests:** File extension filtering (supported vs unsupported vs binary).
- **Unit tests:** Incremental index logic (content_hash + git_blob_oid + parser_version comparison, stale detection, deleted file marking).
- **Performance test:** Index a 500-file fixture repo in under 10 seconds.
- **Regression test:** Existing `hotspots` tests pass without `project_symbols` data.
- **Regression test:** Existing `impact` tests pass unchanged.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M15 applied cleanly to existing ledger.db
- [ ] `changeguard index` on a fixture repo produces non-empty project_symbols
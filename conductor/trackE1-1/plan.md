## Plan: Track E1-1 Full-Project Symbol Index

### Phase 1: Database Schema
- [ ] Task 1.1: Add migration M15 to `src/state/migrations.rs` creating `project_files` table with columns: `id INTEGER PRIMARY KEY`, `file_path TEXT NOT NULL`, `language TEXT`, `content_hash TEXT`, `git_blob_oid TEXT`, `file_size INTEGER`, `mtime_ns INTEGER`, `parser_version TEXT NOT NULL DEFAULT '1'`, `parse_status TEXT NOT NULL DEFAULT 'OK'`, `last_indexed_at TEXT NOT NULL`. Include `UNIQUE(file_path)`. E1-1 owns M15; this must include ALL tables from F0, F1, E1-1, E1-2, E1-3, and E1-4.
- [ ] Task 1.2: In the same migration M15, create `index_metadata` table with columns: `key TEXT PRIMARY KEY`, `value TEXT NOT NULL`. Stored keys: `schema_version`, `index_version`, `tree_sitter_query_version`, `last_git_head`, `last_indexed_at`, `workspace_root`, `path_normalization_version`.
- [ ] Task 1.3: In the same migration M15, create `project_symbols` table with columns: `id INTEGER PRIMARY KEY AUTOINCREMENT`, `file_id INTEGER NOT NULL REFERENCES project_files(id)`, `qualified_name TEXT NOT NULL`, `symbol_name TEXT NOT NULL`, `symbol_kind TEXT NOT NULL`, `visibility TEXT`, `entrypoint_kind TEXT NOT NULL DEFAULT 'INTERNAL'`, `is_public INTEGER DEFAULT 0`, `cognitive_complexity INTEGER`, `cyclomatic_complexity INTEGER`, `line_start INTEGER`, `line_end INTEGER`, `byte_start INTEGER`, `byte_end INTEGER`, `signature_hash TEXT`, `confidence REAL NOT NULL DEFAULT 1.0`, `evidence TEXT`, `last_indexed_at TEXT NOT NULL`, `UNIQUE(file_id, qualified_name, symbol_kind)`. Include indexes on `file_id`, `qualified_name`, `symbol_name`, `symbol_kind`, and `entrypoint_kind`. Note: `cognitive_complexity` and `cyclomatic_complexity` are nullable (`INTEGER`, not `INTEGER NOT NULL`) to match Rust `Option<i32>`.
- [ ] Task 1.4: Update `test_all_tables_exist` in `src/state/migrations.rs` to verify `project_files`, `index_metadata`, and `project_symbols` are created.
- [ ] Task 1.5: Write a new integration test `test_insert_and_query_project_files_symbols` that inserts a file, inserts symbols referencing it, and queries back.

### Phase 2: Domain Types
- [ ] Task 2.1: Create `src/index/project_index.rs` with `ProjectFile` and `ProjectSymbol` structs. `ProjectFile` mirrors `project_files` columns; `ProjectSymbol` mirrors `project_symbols` columns (including `qualified_name`, `byte_start`, `byte_end`, `signature_hash`, `confidence`, `evidence`). Both derive `Serialize`, `Deserialize`, `Clone`, `Debug`. The `Symbol` struct must be extended with `line_start`, `line_end`, `qualified_name`, `byte_start`, `byte_end` fields.
- [ ] Task 2.2: Define `IndexStats` struct with fields: `files_indexed`, `symbols_indexed`, `parse_failures`, `skipped_binary`, `skipped_unsupported`, `duration_ms`. Derive `Serialize` for JSON output.
- [ ] Task 2.3: Define `IndexStatus` struct for `--check` output: `total_files`, `total_symbols`, `stale_files`, `last_indexed_at`. Derive `Serialize`.
- [ ] Task 2.4: Add `pub mod project_index;` to `src/index/mod.rs`.

### Phase 3: Indexing Pipeline
- [ ] Task 3.1: Implement `ProjectIndexer::new(storage, repo_path)` constructor that accepts a `StorageManager` and repo root path.
- [ ] Task 3.2: Implement `ProjectIndexer::discover_files(&self) -> Result<Vec<Utf8PathBuf>>` that uses `gix` to list tracked files, filters by supported language extensions, and excludes binary extensions. Return the list sorted for deterministic ordering.
- [ ] Task 3.3: Implement `ProjectIndexer::index_file(&self, path: &Utf8Path) -> Result<(ProjectFile, Vec<ProjectSymbol>)>` that reads the file, dispatches to `parse_symbols()`, and converts the result into `ProjectFile` and `ProjectSymbol` structs. On parse failure, return a file with `parse_status = 'PARSE_FAILED'`.
- [ ] Task 3.4: Implement `ProjectIndexer::full_index(&self) -> Result<IndexStats>` that clears `project_files` and `project_symbols`, discovers all files, indexes each, and batch-inserts into SQLite (batch size 500). Show `indicatif` progress bar. Return `IndexStats`.
- [ ] Task 3.5: Implement `ProjectIndexer::incremental_index(&self) -> Result<IndexStats>` that queries existing `project_files` for content_hash/git_blob_oid/parser_version and `index_metadata` for last_git_head, compares current values, and only re-indexes changed/new/deleted files. Mark deleted files as `parse_status = 'DELETED'` instead of removing them.
- [ ] Task 3.6: Implement `ProjectIndexer::check_status(&self) -> Result<IndexStatus>` that queries `project_files` for counts and staleness (comparing content_hash, git_blob_oid, parser_version) without modifying the database.
- [ ] Task 3.7: Write unit tests for `discover_files` (extension filtering, binary skipping, empty repo).
- [ ] Task 3.8: Write unit tests for `index_file` (Rust, TypeScript, Python files; parse failure; empty file).
- [ ] Task 3.9: Write integration tests for `full_index` and `incremental_index` using a temp directory with fixture files.

### Phase 4: CLI Command
- [ ] Task 4.1: Create `src/commands/index.rs` with `execute_index(incremental: bool, check: bool)` function. Wire to `ProjectIndexer::full_index()`, `incremental_index()`, or `check_status()` based on flags.
- [ ] Task 4.2: Add `pub mod index;` to `src/commands/mod.rs`.
- [ ] Task 4.3: Wire the `index` subcommand into the CLI argument parser (in `src/cli.rs` or the main clap definition). Add `--incremental` and `--check` flags.
- [ ] Task 4.4: Implement human-readable output for `index` (print stats to stdout). Implement `--json` output using `serde_json::to_string_pretty(&stats)`.
- [ ] Task 4.5: Write CLI integration tests verifying `changeguard index`, `changeguard index --incremental`, and `changeguard index --check` produce expected output.

### Phase 5: Hotspot Fallback
- [ ] Task 5.1: In `src/impact/hotspots.rs`, modify the complexity lookup logic. After querying the `symbols` table for a file's complexity, if the result is 0 or missing, query `project_symbols` (joined with `project_files` on `file_id`) and compute `AVG(cognitive_complexity)` for that file's symbols.
- [ ] Task 5.2: If `project_symbols` also has no data, keep complexity at 0 (graceful degradation). Log a debug-level message.
- [ ] Task 5.3: Write a test that verifies `hotspots` returns non-zero complexity when `project_symbols` has data (joined via `file_id`) but `symbols` does not.
- [ ] Task 5.4: Write a regression test that verifies `hotspots` still works when both `project_symbols` and `symbols` are empty (returns 0 complexity, does not crash). Graceful degradation must also handle the case where `project_files` or `project_symbols` tables do not exist (pre-M15 database).

### Phase 5.5: M15 Migration Ownership
- [ ] Task 5.5: Ensure M15 migration includes ALL required tables: `project_files`, `index_metadata`, `project_symbols`, `project_docs`, and `project_topology`. E1-1 owns this migration. Coordinate with E1-2 and E1-3 for their table additions.

### Phase 6: Performance and Large-Repo Handling
- [ ] Task 6.1: Add batch insert logic using SQLite transactions every 500 files. Ensure the transaction is committed before the next batch.
- [ ] Task 6.2: Add the 10,000 file cap: if `discover_files` returns more than 10,000 files, warn the user and index the first 10,000 only.
- [ ] Task 6.3: Add `indicatif` progress bar showing `Indexing: N/M files...` during full and incremental index.
- [ ] Task 6.4: Write a performance test that indexes a 500-file fixture repo and asserts completion under 10 seconds.
- [ ] Task 6.5: Write a performance test that incremental-indexes 5 changed files and asserts completion under 1 second.
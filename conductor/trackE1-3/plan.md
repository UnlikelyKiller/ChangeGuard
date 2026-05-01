## Plan: Track E1-3 Directory and Module Topology

### Phase 1: Database Schema
- [ ] Task 1.1: Add `project_topology` table creation to migration M15 in `src/state/migrations.rs`. Table columns: `id INTEGER PRIMARY KEY`, `dir_path TEXT NOT NULL`, `role TEXT NOT NULL`, `confidence REAL NOT NULL DEFAULT 1.0`, `evidence TEXT`, `last_indexed_at TEXT NOT NULL`. Include unique index on `dir_path` and index on `role`. E1-1 owns M15; coordinate with E1-1 for the shared migration.
- [ ] Task 1.2: Update `test_all_tables_exist` to verify `project_topology` is created.
- [ ] Task 1.3: Write integration test `test_insert_and_query_project_topology` that inserts topology rows and queries by role and dir_path.

### Phase 2: Domain Types
- [ ] Task 2.1: Create `src/index/topology.rs` with `DirectoryRole` enum: `Source`, `Test`, `Config`, `Infrastructure`, `Documentation`, `Generated`, `Vendor`, `BuildArtifact`. Derive `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, `Eq`. Use `serde(rename_all = "SCREAMING_SNAKE_CASE")`.
- [ ] Task 2.2: Define `DirectoryClassification` struct with fields: `dir_path` (String), `role` (DirectoryRole), `confidence` (f64), `evidence` (String, human-readable reason for the classification).
- [ ] Task 2.3: Define `TopologyIndexStats` struct with fields: `directories_classified` (usize), `unclassified` (usize), `role_counts` (HashMap<DirectoryRole, usize>).
- [ ] Task 2.4: Add `pub mod topology;` to `src/index/mod.rs`.

### Phase 3: Path Pattern Classifier
- [ ] Task 3.1: Implement `classify_by_path(dir_path: &str) -> Option<(DirectoryRole, f64)>` that matches directory paths against known naming conventions. Use ordered matching: `.github/workflows/` -> Infrastructure (1.0), `src/test/` -> Test (1.0), `src/` -> Source (1.0), `tests/` -> Test (1.0), etc. Full list in spec.
- [ ] Task 3.2: Implement parent inheritance: for directories that don't match a pattern directly, check if any parent directory matches and reduce confidence by 0.1 per nesting level (minimum 0.5). E.g., `src/utils/` inherits Source from `src/` at confidence 0.9.
- [ ] Task 3.3: Implement special-case overrides: `.github/workflows/` is Infrastructure (not Config), `src/test/` is Test (not Source), `src/tests/` is Test (not Source).
- [ ] Task 3.4: Write unit tests for `classify_by_path` covering all known patterns, parent inheritance, special cases, and unrecognized paths returning `None`.

### Phase 4: Content Heuristic Classifier
- [ ] Task 4.1: Implement `classify_by_content(dir_path: &str, files: &[&str]) -> Option<(DirectoryRole, f64)>` that examines the file extensions and names in a directory to determine its role. Source if >70% source extensions, Test if >70% test patterns, etc.
- [ ] Task 4.2: Implement file extension classification helpers: `is_source_file(path)`, `is_test_file(path)`, `is_config_file(path)`, `is_doc_file(path)`.
- [ ] Task 4.3: Write unit tests for `classify_by_content` with various file compositions (all source, mixed, empty, all config, etc.).

### Phase 5: Combined Classifier
- [ ] Task 5.1: Implement `classify_directory(dir_path: &str, files: &[&str]) -> DirectoryClassification` that combines path pattern and content heuristic results. Use the higher-confidence result. If path pattern returns confidence >= 0.8, skip content heuristic.
- [ ] Task 5.2: Implement `discover_directories(repo_path: &Utf8Path) -> Result<HashMap<String, Vec<String>>>` that walks the repo using `gix` to list tracked files, extracts unique directory paths, and groups files by directory.
- [ ] Task 5.3: Write unit tests for `classify_directory` covering: path pattern match (skip heuristic), content heuristic fallback, ambiguous directories, deep nesting.

### Phase 6: Topology Indexing
- [ ] Task 6.1: Implement `ProjectIndexer::index_topology(&self) -> Result<TopologyIndexStats>` that discovers directories, classifies each, upserts `project_topology` rows, and returns stats.
- [ ] Task 6.2: Implement directory walking and classification pipeline. Skip empty directories. Accumulate unclassified directories in stats.
- [ ] Task 6.3: Write integration tests for `index_topology` using a temp directory with a multi-directory fixture repo (src/, tests/, .github/workflows/, config/, docs/).
- [ ] Task 6.4: Verify that `index_topology` is deterministic: running twice on the same repo produces identical `project_topology` rows.

### Phase 7: CLI Integration
- [ ] Task 7.1: Modify `execute_index` in `src/commands/index.rs` to call `ProjectIndexer::index_topology()` after source file and doc indexing. Include `TopologyIndexStats` in the output.
- [ ] Task 7.2: Print topology stats in human-readable output (e.g., "Topology: 15 dirs classified - Source: 5, Test: 3, Config: 2, Infrastructure: 2, Documentation: 3").
- [ ] Task 7.3: Write CLI integration test for `changeguard index` verifying topology data is populated.

### Phase 8: Risk Scoring Integration
- [ ] Task 8.1: In `src/impact/analysis.rs`, add a helper function `get_directory_role(storage: &StorageManager, file_path: &str) -> Option<DirectoryRole>` that queries `project_topology` for the file's directory and parent directories.
- [ ] Task 8.2: Modify `analyze_risk()` to call `get_directory_role()` for each changed file. If the role is `Infrastructure` or `Config`, add risk within the Runtime/Config category (max 25 points per expansion plan Section 4.2) and add a corresponding reason string to `risk_reasons`.
- [ ] Task 8.3: If `project_topology` has no data (table empty or missing), skip topology-based risk adjustment (graceful degradation). No warning needed.
- [ ] Task 8.4: Write unit tests verifying: Infrastructure file gets Runtime/Config category risk, Config file gets Runtime/Config category risk, Source file gets no topology-based weight, missing topology data does not affect risk.
- [ ] Task 8.5: Write integration test: `changeguard impact` on a file in `.github/workflows/` produces "Infrastructure change" in risk reasons within the Runtime/Config category.

### Phase 9: Prediction Integration
- [ ] Task 9.1: In `src/verify/predict.rs`, add a helper function `find_test_candidates(storage: &StorageManager, source_file: &str) -> Vec<String>` that queries `project_topology` for `Test` directories and then queries `project_symbols` (or `project_modules`) for test files matching the source file's name.
- [ ] Task 9.2: Add topology-based test candidates to the verification prediction output with reason `"Topology: test directory candidate"`.
- [ ] Task 9.3: If `project_topology` has no data, skip topology-based prediction (graceful degradation).
- [ ] Task 9.4: Write unit tests for `find_test_candidates` with fixture data.
- [ ] Task 9.5: Write integration test: `changeguard verify` on a changed source file predicts test files in `Test` directories as verification targets.
# Specification: Track E1-3 Directory and Module Topology

## Overview

This track adds directory-role classification to ChangeGuard. It labels project directories by their architectural role (Source, Test, Config, Infrastructure, etc.) using naming conventions and content heuristics. The classifications are stored in a `project_topology` table (Migration M15, shared with E1-1 and E1-2) and integrated with risk scoring and verification prediction. Files in Infrastructure and Config directories receive +20 risk weight; test files in Test directories become candidate verification targets for changed source files.

This track depends on E1-1 for the `index` command infrastructure and the `ProjectIndexer` pattern.

## Components

### 1. Database Migration M15 - Part C: `project_topology` Table (`src/state/migrations.rs`)

Add the `project_topology` table to the same M15 migration. **E1-1 owns M15**; this track adds its table to that shared migration.

```sql
CREATE TABLE IF NOT EXISTS project_topology (
    id              INTEGER PRIMARY KEY,
    dir_path        TEXT NOT NULL,
    role            TEXT NOT NULL,
    confidence      REAL NOT NULL DEFAULT 1.0,
    evidence        TEXT,
    last_indexed_at TEXT NOT NULL,
    UNIQUE(dir_path)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_project_topology_dir_path
    ON project_topology(dir_path);
CREATE INDEX IF NOT EXISTS idx_project_topology_role
    ON project_topology(role);
```

- `id`: Integer primary key for stable references.
- `dir_path`: Repository-relative directory path (e.g., `src`, `tests`, `.github/workflows`). Always uses forward slashes.
- `role`: One of the `DirectoryRole` enum values (see below).
- `confidence`: Float between 0.0 and 1.0. Per the expansion plan, every extracted fact must carry a confidence score. Directories that match naming conventions exactly get 1.0; ambiguous directories get reduced confidence (e.g., 0.6).
- `evidence`: Human-readable reason for the classification (e.g., `"contains 15 .rs files, 0 test files"`). Per the expansion plan, every extracted fact must carry an evidence string.
- `last_indexed_at`: ISO 8601 timestamp of the last indexing run.

This track depends on E1-1's `project_files` table for file discovery and classification.

### 2. Directory Role Enum (`src/index/topology.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectoryRole {
    Source,
    Test,
    Config,
    Infrastructure,
    Documentation,
    Generated,
    Vendor,
    BuildArtifact,
}
```

**Tie-breaking order** (for ambiguous directories where multiple roles could apply): Source > Test > Config > Infrastructure > Documentation > Generated > Vendor > BuildArtifact.

Each role corresponds to a classification rule set:

**Source:**
- `src/`, `src/main/`, `lib/`, `app/`, `pkg/`, `internal/`
- Directories containing primarily source files (`.rs`, `.ts`, `.py`)
- Confidence: 1.0 for exact name matches; 0.8 for heuristic matches

**Test:**
- `tests/`, `test/`, `spec/`, `specs/`, `__tests__/`, `test_utils/`
- Directories containing primarily test files (matching `*_test.*`, `*_spec.*`, `test_*.*`)
- Confidence: 1.0 for exact name matches; 0.8 for heuristic matches

**Config:**
- `config/`, `configs/`, `conf/`, `.config/`
- Directories containing primarily config files (`.toml`, `.yaml`, `.json` config files, `.env.example`)
- Confidence: 1.0 for exact name matches; 0.7 for heuristic matches

**Infrastructure:**
- `.github/workflows/`, `.github/`, `.gitlab/`, `.circleci/`, `ci/`, `deploy/`, `deployment/`, `terraform/`, `k8s/`, `kubernetes/`, `helm/`, `docker/`
- Confidence: 1.0

**Documentation:**
- `docs/`, `doc/`, `documentation/`
- Directories containing primarily `.md` files
- Confidence: 1.0 for exact name matches; 0.7 for heuristic matches

**Generated:**
- `dist/`, `build/`, `out/`, `output/`, `.generated/`, `__generated__/`
- Confidence: 1.0

**Vendor:**
- `vendor/`, `third_party/`, `thirdparty/`, `external/`, `deps/`
- Confidence: 1.0

**BuildArtifact:**
- `target/`, `node_modules/`, `.gradle/`, `.cache/`, `.cargo/registry/`
- Confidence: 1.0

### 3. Directory Classifier (`src/index/topology.rs`)

The classifier examines each directory in the repository and assigns a role based on two strategies:

**Strategy 1: Path pattern matching (primary)**
- Match the directory path against the known naming conventions listed above.
- If a directory path matches exactly (e.g., `src/` matches Source), assign with confidence 1.0.
- If a parent directory matches (e.g., `src/utils/` is under `src/`), assign the parent's role with confidence reduced by 0.1 per level of nesting (minimum 0.5).

**Strategy 2: Content heuristic (fallback)**
- If no path pattern matches, examine the files in the directory:
  - If >70% of files are source files (`.rs`, `.ts`, `.py`): Source (0.6)
  - If >70% of files match test patterns (`*_test.*`, `*_spec.*`): Test (0.6)
  - If >70% of files are config files (`.toml`, `.yaml`, `.json`): Config (0.6)
  - If the directory contains `Dockerfile`, `.github/`, or CI YAML: Infrastructure (0.7)
  - If >70% of files are `.md` files: Documentation (0.6)
  - Otherwise: skip (do not classify)

**Classification algorithm:**
1. Walk all directories in the repository (using `gix` to list tracked files, then extract unique directory paths).
2. For each directory, try path pattern matching first.
3. If no match or confidence < 0.8, try content heuristic.
4. Use the higher-confidence result.
5. For ambiguous directories (e.g., `examples/` could be Source or Test), assign the best-matching role with reduced confidence (0.5-0.7).

**Special cases:**
- `.github/workflows/` is Infrastructure (not Config, even though it contains YAML).
- `src/test/` or `src/tests/` is Test (not Source), because the `test` component overrides.
- Monorepos with multiple top-level source directories: classify each independently.

### 4. Topology Indexing Integration (`src/index/project_index.rs`)

Extend `ProjectIndexer` with topology indexing methods:

```rust
pub fn index_topology(&self) -> Result<TopologyIndexStats>
```

**`TopologyIndexStats`:**
```rust
pub struct TopologyIndexStats {
    pub directories_classified: usize,
    pub unclassified: usize,
    pub role_counts: HashMap<DirectoryRole, usize>,
}
```

The `index_topology` method:
1. Walks all directories in the repository.
2. Classifies each directory using the classifier.
3. Upserts `project_topology` rows.
4. Returns stats.

### 5. `index` Command Integration (`src/commands/index.rs`)

Extend `execute_index` to call `ProjectIndexer::index_topology()` after source file and doc indexing. Print topology stats (e.g., "Topology: 15 directories classified (5 Source, 3 Test, 2 Config, 2 Infrastructure, 3 Documentation)").

### 6. Risk Scoring Integration (`src/impact/analysis.rs`)

Modify the `analyze_risk()` function to query `project_topology` for the directory role of each changed file:

- If a file is in a directory classified as `Infrastructure`, its risk contribution falls within the Runtime/Config category (max 25 points per expansion plan Section 4.2). Add `"Infrastructure change: {dir_path}"` to `risk_reasons`.
- If a file is in a directory classified as `Config`, its risk contribution falls within the Runtime/Config category (max 25 points per expansion plan Section 4.2). Add `"Configuration change: {dir_path}"` to `risk_reasons`.
- This parallels the existing protected-path elevation, which also adds risk weight for certain paths. The `project_topology` role is an additional signal.

**Integration approach:**
- After computing the base risk score, determine the file's directory path.
- Query `project_topology` for the role of that directory (and parent directories if no exact match).
- If the role is `Infrastructure` or `Config`, add the risk weight.
- If `project_topology` has no data (pre-M15 database or no index run), skip this step (graceful degradation).

### 7. Prediction Integration (`src/verify/predict.rs`)

Modify verification prediction to use topology data:

- When predicting verification targets for a changed source file, look for test files in directories classified as `Test` that correspond to the source file's path.
- Example: `src/parser.rs` changed -> look for `tests/parser_test.rs` or `src/parser_test.rs` in `Test` directories.
- Add these as prediction candidates with reason `"Topology: test directory candidate"`.

## Constraints

- **No network access:** Classification is based on local file paths and contents only.
- **No `.gitignore` bypass:** Only classify directories containing tracked files (respecting `.gitignore`).
- **Graceful degradation:** If `project_topology` is empty, risk scoring and prediction work without topology data. No warnings are emitted for empty topology data (it simply means `index` has not been run).
- **Deterministic classification:** The same repository always produces the same topology. No randomness or LLM-based classification.
- **Performance:** Topology classification must complete in under 2 seconds for a repository with 1,000 directories.

## Edge Cases

- **Ambiguous directories:** `examples/` could be Source or Test. Assign `Source` with confidence 0.5. The classifier should use a deterministic tie-breaking rule: Source > Test > Config > Infrastructure > Documentation > Generated > Vendor > BuildArtifact for ambiguous cases.
- **Monorepos with multiple top-level source directories:** Classify each top-level directory independently. E.g., `services/auth/src/` is Source, `services/auth/tests/` is Test.
- **Directories containing both source and test files:** Classify based on majority (>50% source files -> Source, >50% test files -> Test). If neither majority is clear, use the path pattern match.
- **Empty directories:** Skip. Only classify directories that contain at least one tracked file.
- **Deeply nested directories:** `src/module/submodule/deep/` inherits Source from `src/` with confidence reduced by 0.1 per level (minimum 0.5).
- **`target/` and `node_modules/`:** Classified as `BuildArtifact`. These are typically in `.gitignore` and will not appear in tracked files. If they do appear, classify them correctly.
- **`.github/workflows/`:** Classified as `Infrastructure`, not `Config`. The path pattern match for `.github/workflows/` takes precedence over any content heuristic.
- **`src/test/` or `src/tests/`:** Classified as `Test`. The `test` component in the path overrides the `src` prefix.

## Acceptance Criteria

1. `changeguard index` populates `project_topology` with classified directories for all directories containing tracked files.
2. Files in `Infrastructure` directories contribute risk within the Runtime/Config category (max 25 points, per expansion plan Section 4.2) in `changeguard impact`.
3. Files in `Config` directories contribute risk within the Runtime/Config category (max 25 points, per expansion plan Section 4.2) in `changeguard impact`.
4. `changeguard verify` identifies test files in `Test` directories as candidate verification targets for changed source files.
5. Topology classification is deterministic: running `index` twice on the same repo produces identical results.
6. Directories with ambiguous roles receive reduced confidence scores (0.5-0.7).
7. Risk scoring and prediction work correctly when `project_topology` is empty (graceful degradation).

## Verification Gate

- **Unit tests:** `DirectoryRole` enum serialization and deserialization.
- **Unit tests:** Directory classifier correctly classifies known path patterns (`src/` -> Source, `tests/` -> Test, `.github/workflows/` -> Infrastructure, etc.).
- **Unit tests:** Content heuristic correctly classifies directories by file composition.
- **Unit tests:** Ambiguous directories receive reduced confidence.
- **Unit tests:** Nested directories inherit roles with reduced confidence.
- **Integration test:** `changeguard index` on a multi-directory fixture repo populates `project_topology` correctly.
- **Integration test:** `changeguard impact` on a file in `.github/workflows/` produces an "Infrastructure change" risk reason within the Runtime/Config category.
- **Integration test:** `changeguard verify` predicts test files as verification targets for changed source files.
- **Regression test:** Existing `impact` and `verify` tests pass without `project_topology` data.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M15 applied cleanly to existing ledger.db
- [ ] `changeguard index` on a fixture repo produces non-empty project_symbols
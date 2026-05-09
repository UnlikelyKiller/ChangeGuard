# ChangeGuard Expansion Plan: System Intelligence

This document is the implementation roadmap for expanding ChangeGuard from **Transactional Change Intelligence** (current) to **System-Wide Architectural Intelligence**. It is grounded in the actual codebase as of Milestone L completion and avoids breaking any existing feature or function.

The expansion follows a four-layer model. Each layer builds on the one before it. No layer requires a radical schema rewrite; all changes extend the existing `ledger.db`, `ImpactPacket`, and CLI surface incrementally.

---

## 0. Current State Summary

ChangeGuard (post-Milestone L) provides:

- **Git scanning and change classification** (`scan`, `watch`)
- **Symbol, import/export, runtime usage, and complexity extraction** for Rust, TypeScript, and Python (`impact`)
- **Temporal coupling** from git history with exponential decay weighting (`impact --all-parents`)
- **Hotspot ranking** from normalized complexity × frequency (`hotspots`)
- **Predictive verification** using structural imports, historical imports, and temporal coupling (`verify`)
- **Risk scoring** with protected-path elevation (`impact`)
- **Federated sibling schema** export, scan, and cross-repo impact checks (`federate`)
- **Ledger transaction lifecycle** (start, commit, rollback, atomic, note, drift detection, reconciliation, adoption)
- **Tech stack enforcement** (NO rules at start time, commit validators)
- **FTS5 search**, **MADR ADR export**, **token-level provenance** (data layer only)
- **Gemini prompt modes** (analyze, suggest, review_patch, narrative) with token budgeting and fallback
- **LSP daemon** (diagnostics, hover, CodeLens) behind a feature flag
- **Secret redaction** before persistence and Gemini submission
- **Cross-platform** Windows/WSL/Linux support with process policy enforcement

### Known Gaps (From Deep Dive)

| Gap | Area | Impact |
|-----|------|--------|
| Hotspot complexity is 0 without a prior `impact` run | Intelligence | Hotspots become meaningless on fresh repos |
| No per-symbol hotspot identification | Intelligence | Hotspots are file-level only; cannot pinpoint risky functions |
| Federated dependency matching is text-based substring | Federation | False positives (e.g., "api" matches "map_item") |
| Runtime usage collected but unused in risk scoring | Intelligence | Env vars and config keys don't affect risk |
| Verification gate not enforced at commit time | Ledger | Categories marked as requiring verification accept commits without it |
| `ledger track` CLI command not implemented | Ledger | Token provenance data layer exists but no way to manually attach it |
| LSP diagnostics are position (0,0) only | LSP | Diagnostics are file-level, not symbol-level |
| Token budget hardcoded at 409,600 chars, inconsistent with config | Gemini | Config `context_window` field is unused |
| No CI/CD workflow awareness | Intelligence | Cannot warn about changes to GitHub Actions, Jenkins, etc. |
| No test-to-symbol mapping | Intelligence | Cannot predict which tests verify which symbols |
| `scan` shells out to `git diff HEAD` instead of using gix natively | Git | Inconsistent with the gix-first principle; binary file risk |
| Unused DB columns (`operation_id`, `issue_ref`, `snapshot_id`, `tree_hash` in `ledger_entries`) | Ledger | Schema bloat without functional value |

---

## 1. Expansion Vision

ChangeGuard currently answers: **"What changed, and what might it affect?"**

The expansion answers: **"What changed, what does it mean for the system, and how do we verify it confidently?"**

This requires four capabilities the current system lacks:

1. **Structural Bearings** — A new developer or agent entering a repo should get an immediate map of where things are, what the entry points are, and which modules are heaviest. Currently, ChangeGuard only extracts symbols from *changed files*. The expansion indexes the full project structure.

2. **Behavioral Mapping** — ChangeGuard detects *that* a function changed, but not *what routes, data models, or API contracts* that function serves. The expansion maps external surfaces to internal logic.

3. **Observability Wiring** — When a developer changes error handling or logging, ChangeGuard doesn't know. The expansion detects observability patterns and flags changes that reduce system visibility.

4. **Safety Context** — ChangeGuard has protected paths and commit validators, but it doesn't know which tests guard which symbols, what CI gates exist, or which environment variables are required. The expansion closes these gaps.

---

## 2. Non-Negotiable Constraints

All expansion work must respect these constraints. Violations are blocking.

1. **Local-first always.** No network calls, no cloud dependencies, no required Python runtime. All analysis happens on the user's machine.
2. **Single binary.** The expansion remains within the `changeguard` crate. New tree-sitter queries are in-tree. No new build dependencies without a concrete phase requirement.
3. **Graceful degradation.** If a repo lacks a README, tests, CI config, or observability code, ChangeGuard must still function for the remaining layers. Missing data is a visible warning, never a crash.
4. **Deterministic over speculative.** If a route or test mapping cannot be identified with high confidence, label it `POTENTIAL_ROUTE` or `INFERRED_TEST` rather than guessing. Never present speculation as fact. Every extracted fact must carry a `confidence` score and `evidence` string.
5. **Backward-compatible schema.** All new `ledger.db` migrations are additive. No columns are dropped, no existing semantics change. New columns default to sensible values.
6. **Existing CLI stability.** No existing command's flags, arguments, or output format changes in a breaking way. New flags and subcommands are additive.
7. **No performance regression.** The `impact` command must complete in under 5 seconds for a repo with 200 changed files. The `hotspots` command must complete in under 10 seconds for a repo with 10,000 commits. If an expansion phase risks these targets, it must include a lazy/optional path.
8. **Stable identity.** No downstream table may rely solely on `file_path + symbol_name` for joins. All identity references use integer foreign keys (`file_id`, `symbol_id`) with fallback text fields only for unresolved cases. This prevents false matches from renames, duplicate names across modules, and cross-language name collisions.

---

## 3. Foundation Tracks (Pre-E1)

These tracks establish the identity and invalidation infrastructure that all E1-E4 tracks depend on.

### Track F0: Stable Project Index Identity

**Goal:** Create `project_files` as the canonical identity table for all indexed files. All other tables reference `file_id` rather than `file_path` strings.

**Deliverables:**

- `project_files` table (Migration M15):

```sql
CREATE TABLE project_files (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,          -- repo-relative, forward slashes, no ./
    language TEXT,                     -- "Rust", "TypeScript", "Python", etc.
    content_hash TEXT,                 -- BLAKE3 hash of file content
    git_blob_oid TEXT,                -- git object ID for this file at HEAD
    file_size INTEGER,
    mtime_ns INTEGER,                -- nanosecond mtime for incremental detection
    parser_version TEXT NOT NULL DEFAULT '1',
    parse_status TEXT NOT NULL DEFAULT 'OK',  -- OK, PARSE_FAILED, UNSUPPORTED, EMPTY
    last_indexed_at TEXT NOT NULL,
    UNIQUE(file_path)
);
```

- `index_metadata` table (Migration M15):

```sql
CREATE TABLE index_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

  Stored keys: `schema_version`, `index_version`, `tree_sitter_query_version`, `last_git_head`, `last_indexed_at`, `workspace_root`, `path_normalization_version`.

- Windows path normalization: store all paths as repo-relative with forward slashes. On case-insensitive filesystems, lowercase paths. Never store absolute user paths.

**Acceptance criteria:**

- `project_files` can be created and queried
- Paths stored are always repo-relative with forward slashes
- Two files with the same name in different directories have different `id` values

**Verification gate:**

- Unit tests for `project_files` CRUD
- Path normalization tests for Windows backslash, mixed separators, `./` prefix

---

### Track F1: Index Invalidation and File Discovery

**Goal:** Ensure the project index is never silently stale. Use a three-part invalidation key: content hash + git blob OID + parser version.

**Current state:** The initial expansion plan relied on mtime-only incremental indexing. Mtime is unreliable on Windows (git operations, checkout, sync tools can produce misleading mtimes). Parser changes should invalidate stale results even when source files did not change.

**Deliverables:**

- File discovery: walk the repo using the existing `ignore` crate (already a dependency), respecting `.gitignore`. Skip binary files by extension list.
- Invalidation logic in `changeguard index`:
  1. Query `index_metadata` for `last_git_head`. If HEAD has changed, mark all files for re-validation.
  2. For each source file, compare `content_hash` (BLAKE3 of file content) and `parser_version` against stored values.
  3. If either differs, re-parse and update `project_files` and dependent tables.
  4. If `parser_version` has changed globally (e.g., tree-sitter query update), force full re-index.
- `changeguard index --check` flag: report stale files without re-indexing.
- Progress reporting via `indicatif` (already a dependency).

**Edge cases:**

- Very large repos (>10,000 source files): batch inserts every 500 files, show progress bar.
- Files deleted between index runs: mark as `parse_status = 'DELETED'`, do not remove from `project_files` (preserves historical references).
- Files added between index runs: insert new `project_files` entries.
- `.gitignore` changes: re-scan all files if `.gitignore` mtime changed since last index.

**Acceptance criteria:**

- Changing a file's content triggers re-index on next `changeguard index`
- Changing `parser_version` in `index_metadata` triggers full re-index
- `changeguard index --check` reports stale files without modifying the database
- Deleted files are marked, not removed

**Verification gate:**

- Integration test: modify a fixture file, run `index --check`, verify it reports the file as stale
- Integration test: run `index`, modify `parser_version`, verify all files are re-indexed

---

## 4. The Four Layers

### Phase E1: Structural Bearings

**Intent:** Eliminate the "blind dive." When an agent or new developer enters a repo, ChangeGuard should provide an immediate architectural map without requiring them to read every file.

The current system only indexes *changed* files during `impact`. The expansion adds a standalone `index` command that builds a full project map and stores it in SQLite, making it available to all downstream features (hotspots, prediction, LSP, federation).

#### Track E1-1: Full-Project Symbol Index

**Goal:** Extract symbols, imports, exports, and complexity for *all* source files in the repo, not just changed ones. Store results in `project_symbols` (a separate, always-current index). The existing `symbols` table used by `impact` remains unchanged.

**Current state:** Symbol extraction (`src/index/languages/`) only runs during `impact` on changed files. The `symbols` table in SQLite only contains files from the last impact run. The `hotspots` command queries `symbols` for complexity, getting 0 for unindexed files.

**Deliverables:**

- `changeguard index` command that scans all supported source files and populates `project_files`, `project_symbols`
- `project_symbols` table (Migration M15):

```sql
CREATE TABLE project_symbols (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES project_files(id),
    qualified_name TEXT NOT NULL,      -- e.g., "crate::module::function_name"
    symbol_name TEXT NOT NULL,          -- short name, e.g., "function_name"
    symbol_kind TEXT NOT NULL,          -- Function, Method, Class, Struct, etc.
    visibility TEXT,                     -- public, private, crate, etc.
    entrypoint_kind TEXT NOT NULL DEFAULT 'INTERNAL',  -- ENTRYPOINT, HANDLER, PUBLIC_API, TEST, INTERNAL
    cognitive_complexity INTEGER,
    cyclomatic_complexity INTEGER,
    line_start INTEGER,
    line_end INTEGER,
    byte_start INTEGER,
    byte_end INTEGER,
    signature_hash TEXT,               -- BLAKE3 of the symbol's signature text
    confidence REAL NOT NULL DEFAULT 1.0,
    last_indexed_at TEXT NOT NULL,
    UNIQUE(file_id, qualified_name, symbol_kind)
);

CREATE INDEX idx_project_symbols_file ON project_symbols(file_id);
CREATE INDEX idx_project_symbols_qualified ON project_symbols(qualified_name);
CREATE INDEX idx_project_symbols_name ON project_symbols(symbol_name);
CREATE INDEX idx_project_symbols_kind ON project_symbols(symbol_kind);
CREATE INDEX idx_project_symbols_entrypoint ON project_symbols(entrypoint_kind);
```

- The `symbols` table currently used by `impact` is unchanged; `project_symbols` is a separate, always-current index
- `changeguard hotspots` will fall back to `project_symbols` when `symbols` has no data for a file, fixing the "0 complexity on fresh repos" gap
- Incremental indexing: use content_hash + parser_version comparison from F1, not mtime alone
- Batch insert: 500 rows per SQLite transaction

**Edge cases:**

- Very large repos (>10,000 source files): streaming inserts, batch commits, progress bar
- Parse failures: mark file as `parse_status = 'PARSE_FAILED'` in `project_files`, continue to next file
- Mixed-language repos: dispatch per-language, accumulate all results, report unsupported files as `UNSUPPORTED`
- Binary files: skip by extension list (`.png`, `.jpg`, `.woff`, `.ico`, etc.)
- Duplicate function names in different modules: `qualified_name` distinguishes them (e.g., `crate::module_a::run` vs `crate::module_b::run`)
- Methods on different structs with the same name: `qualified_name` includes the struct name where determinable

**Acceptance criteria:**

- `changeguard index` populates `project_files` and `project_symbols` for all supported source files
- `changeguard hotspots` on a fresh repo (no prior `impact` run) returns meaningful complexity scores from the project index
- Incremental re-index takes < 1 second when only 5 files changed
- Parse failures are surfaced as warnings, never crashes
- Renaming a file does not create duplicate stale symbols after re-index

**Verification gate:**

- Integration test: init + index + hotspots on a fixture repo produces non-zero complexity scores
- Unit tests for `project_files` and `project_symbols` CRUD
- Performance test: index a 500-file fixture repo in < 10 seconds
- Rename test: rename a fixture file, re-index, verify no stale entries

---

#### Track E1-2: README and Documentation Ingestion

**Goal:** Parse `README.md` and other documentation files to ground analysis in the project's stated mission and structure.

**Current state:** No documentation parsing exists. ChangeGuard treats `.md` files as opaque.

**Deliverables:**

- Markdown parser (using `pulldown-cmark` crate, version `0.13.3`) for `README.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md`, and files referenced from `README.md`
- `project_docs` table (Migration M15):

```sql
CREATE TABLE project_docs (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES project_files(id),
    title TEXT,
    summary TEXT,                      -- deterministic extraction only (see below)
    sections JSON,                     -- [{title, level, line_start}]
    code_blocks JSON,                  -- [{language, line_start, line_end}]
    internal_links JSON,               -- [{target, line_start}]
    last_indexed_at TEXT NOT NULL,
    UNIQUE(file_id)
);
```

- **Summary is extractive, not generative.** It is defined as: first non-empty paragraph under the title + top-level heading outline + detected code block languages. No LLM or model-generated text.
- The `ask` command will include the project summary in Gemini system prompts when available
- The `audit` command will surface "No README found" as a project health warning

**Edge cases:**

- No README: graceful skip, warning only
- Malformed markdown: extract what's possible, mark file as `parse_status = 'PARSE_FAILED'`
- Very large README (>100KB): extract title and first 500 lines only
- Binary/encoded content in code blocks: skip code blocks with non-UTF-8 content

**Acceptance criteria:**

- `changeguard index` populates `project_docs` when `README.md` exists
- `changeguard ask --narrative` includes project summary in system prompt when `project_docs` has data
- Missing README is a visible warning, not an error
- Summary contains only deterministic extractions, no generated text

**Verification gate:**

- Fixture test: README with sections, links, and code blocks is parsed correctly
- Fixture test: repo without README produces a warning and continues

---

#### Track E1-3: Directory and Module Topology

**Goal:** Label project directories by role (source, test, config, infrastructure, docs) based on naming conventions and contents. Expose this to risk scoring and prediction.

**Current state:** ChangeGuard has no concept of directory role. It treats all files equally.

**Deliverables:**

- Directory classifier that assigns roles: `Source`, `Test`, `Config`, `Infrastructure`, `Documentation`, `Generated`, `Vendor`, `BuildArtifact`
- Classification rules: `src/`, `lib/` → Source; `tests/`, `test/`, `spec/` → Test; `.github/workflows/` → Infrastructure; `docs/`, `doc/` → Documentation; `target/`, `node_modules/`, `dist/` → BuildArtifact; `vendor/`, `third_party/` → Vendor
- `project_topology` table (Migration M15):

```sql
CREATE TABLE project_topology (
    id INTEGER PRIMARY KEY,
    dir_path TEXT NOT NULL,
    role TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,
    evidence TEXT,                      -- e.g., "contains 15 .rs files, 0 test files"
    last_indexed_at TEXT NOT NULL,
    UNIQUE(dir_path)
);
```

- Integration with risk scoring: files in `Infrastructure` and `Config` directories receive risk in the API Surface category (max 35 points, see Section 4.6)
- Integration with prediction: test files in `Test` directories are candidate verification targets for changed source files in `Source` directories

**Edge cases:**

- Ambiguous directories (e.g., `examples/` could be Source or Test): assign role with reduced confidence and explain in `evidence`
- Monorepos with multiple top-level source directories: classify each independently
- Directories that contain both source and test files: classify based on majority, with `evidence` explaining the split

**Acceptance criteria:**

- `changeguard index` populates `project_topology` with classified directories
- Risk scoring gives Infrastructure/Config category weight
- Prediction identifies test files as candidates for verification

**Verification gate:**

- Fixture test: classify a multi-directory project correctly
- Risk scoring test: verify Infrastructure/Config files receive elevated risk

---

#### Track E1-4: Entry Point Identification

**Goal:** Trace the "first breath" of the application — main functions, HTTP handlers, CLI entry points, test entry points — so that downstream features can reason about call chains and blast radius.

**Current state:** Symbol extraction finds `function_item`, `struct_item`, etc., but does not distinguish entry points from internal functions.

**Deliverables:**

- Entry point detection via tree-sitter queries:
  - Rust: `fn main()`, `#[tokio::main]`, `#[actix_web::main]`, `#[test]` functions
  - TypeScript: files named `main.ts`, `index.ts`, `server.ts`, exported default functions from entry modules
  - Python: `if __name__ == "__main__":` blocks, `app = FastAPI()` / `app = Flask()` declarations
- `entrypoint_kind` column on `project_symbols` (already in E1-1 schema): values `ENTRYPOINT`, `HANDLER`, `PUBLIC_API`, `TEST`, `INTERNAL` (default)
  - **Do NOT label all `pub fn` as `HANDLER` when no entry points are found.** Public library functions that are not route handlers should be labeled `PUBLIC_API`.
- Integration with `impact`: changed entry points receive risk in the API Surface category (see Section 4.6)
- Integration with LSP: entry points shown in CodeLens as "Entry Point" / "Handler" / "Test" / "Public API"

**Edge cases:**

- No identifiable entry points (library crate): label all `pub fn` as `PUBLIC_API` with `confidence = 0.7` and `evidence = "no entry point found; all public symbols labeled as public API"`
- Macro-generated entry points (`#[derive(...)]`): skip, mark as `INTERNAL`
- Multiple entry points in one file: mark all that match

**Acceptance criteria:**

- `changeguard index` labels entry points in Rust, TypeScript, and Python files
- `changeguard impact` gives elevated risk to changed entry points
- Library crates label public functions as `PUBLIC_API`, not `HANDLER`
- LSP CodeLens shows "Entry Point" for labeled symbols (when daemon feature is enabled)

**Verification gate:**

- Fixture test: Rust binary crate, TypeScript server, Python script all get entry points labeled
- Fixture test: Rust library crate labels `pub fn` as `PUBLIC_API`, not `HANDLER`
- Risk scoring test: entry point changes receive elevated risk

---

### Phase E2: Behavioral Mapping

**Intent:** Map the "surface area" (APIs, routes, handlers) to the "internal organs" (business logic, data models). This allows ChangeGuard to answer "if I change this internal function, which external endpoints are affected?"

#### Track E2-1: Call Graph and Structural Coupling

**Goal:** Build a lightweight call graph from tree-sitter ASTs, mapping which symbols call which other symbols within the project. Store as edges in `structural_edges`.

**Current state:** The `imports` and `exports` fields on `ChangedFile` show what a file *imports* and *exports*, but not which internal symbols call which others. Temporal coupling is statistical, not structural.

**Deliverables:**

- Call extraction via tree-sitter queries:
  - Rust: `call_expression` nodes where callee resolves to a known function
  - TypeScript: `call_expression` and `new_expression` nodes
  - Python: `call` nodes in function bodies
- `structural_edges` table (Migration M16):

```sql
CREATE TABLE structural_edges (
    id INTEGER PRIMARY KEY,
    caller_symbol_id INTEGER REFERENCES project_symbols(id),
    callee_symbol_id INTEGER REFERENCES project_symbols(id),
    caller_file_id INTEGER NOT NULL REFERENCES project_files(id),
    callee_file_id INTEGER REFERENCES project_files(id),
    unresolved_callee TEXT,                 -- text name when callee not in project_symbols
    call_kind TEXT NOT NULL DEFAULT 'DIRECT',  -- DIRECT, METHOD_CALL, TRAIT_DISPATCH, DYNAMIC, EXTERNAL
    resolution_status TEXT NOT NULL DEFAULT 'RESOLVED',  -- RESOLVED, AMBIGUOUS, UNRESOLVED
    confidence REAL NOT NULL DEFAULT 1.0,
    evidence TEXT,                          -- e.g., "call_expression at line 42"
    last_indexed_at TEXT NOT NULL
);

CREATE INDEX idx_structural_edges_callee ON structural_edges(callee_symbol_id);
CREATE INDEX idx_structural_edges_caller ON structural_edges(caller_symbol_id);
```

- Unresolved calls (dynamic dispatch, trait objects, FFI): stored with `resolution_status = 'UNRESOLVED'` or `'AMBIGUOUS'`, never presented as confirmed
- Integration with `impact`: when a symbol changes, query `structural_edges` for all callers → add structured risk reason
- Integration with `verify`: use `structural_edges` as an additional prediction source beyond temporal coupling and import-based prediction

**Edge cases:**

- Dynamic dispatch (trait objects, function pointers, callback patterns): mark as `DYNAMIC` with `confidence = 0.3` and `evidence = "dynamic dispatch"`
- Cross-language calls (e.g., Python calling Rust via FFI): mark as `EXTERNAL` with `confidence = 0.5`
- Recursive calls: store but don't follow recursively during query
- Very large call graphs (>50K edges per file): cap, prioritize public-symbol edges, mark as `resolution_status = 'CAPPED'`
- Generics/monomorphization: store the generic call, not each monomorphization

**Acceptance criteria:**

- `changeguard index` populates `structural_edges` for supported languages
- `changeguard impact` includes structurally-coupled callers in risk reasons when applicable
- `changeguard verify` includes structurally-predicted files in verification plans
- Dynamic/unresolved calls are labeled as such, never as confirmed

**Verification gate:**

- Fixture test: a Rust project with `main()` → `helper()` → `internal()` gets edges with resolution_status = 'RESOLVED'
- Impact test: changing `internal()` produces a risk reason mentioning `helper` as a caller
- Verify test: changing `internal()` predicts `helper` as a verification target

---

#### Track E2-2: Framework-Aware Route Mapping

**Goal:** Detect web framework routing and map URL patterns to handler functions.

**Current state:** No framework awareness. Changed files in a web server project are treated identically to changed files in a library project.

**Deliverables:**

- Route extraction via tree-sitter queries and pattern matching:
  - **Rust Actix**: `#[actix_web::get("/path")]`, `#[actix_web::post("/path")]` attributes on handler functions
  - **Rust Axum**: `Router::new().route("/path", get(handler))` method chains; `Router::nest("/prefix", router)` groups. Detect `.route()`, `.nest()`, `.layer()` call chains.
  - **Rust Rocket**: `#[rocket::get("/path")]` attributes
  - **TypeScript Express**: `app.get("/path", handler)`, `app.post("/path", handler)`, `router.method("/path", handler)`, `app.use("/prefix", router)` mounts
  - **Python FastAPI**: `@app.get("/path")`, `@app.post("/path")` decorators on handler functions
  - **Python Flask**: `@app.route("/path")`, `@app.route("/path", methods=["GET", "POST"])` decorators
- `api_routes` table (Migration M16):

```sql
CREATE TABLE api_routes (
    id INTEGER PRIMARY KEY,
    method TEXT NOT NULL,               -- GET, POST, PUT, DELETE, PATCH, etc.
    path_pattern TEXT NOT NULL,          -- "/users/:id" or "/users/{id}"
    handler_symbol_id INTEGER REFERENCES project_symbols(id),
    handler_file_id INTEGER NOT NULL REFERENCES project_files(id),
    handler_symbol_name TEXT,            -- fallback text name when symbol_id is null
    framework TEXT NOT NULL,             -- actix, axum, rocket, express, fastapi, flask
    route_source TEXT NOT NULL DEFAULT 'DECORATOR',  -- DECORATOR, ROUTER_CHAIN, APP_METHOD, MOUNTED_ROUTER
    mount_prefix TEXT,                    -- e.g., "/api" for nested routers
    is_dynamic INTEGER DEFAULT 0,        -- 1 if path contains variables
    route_confidence REAL NOT NULL DEFAULT 1.0,
    evidence TEXT,                        -- e.g., "#[get(\"/users\")] on list_users"
    last_indexed_at TEXT NOT NULL
);

CREATE INDEX idx_api_routes_handler ON api_routes(handler_symbol_id);
CREATE INDEX idx_api_routes_path ON api_routes(path_pattern);
```

- Integration with risk scoring: changed route handlers receive risk in the API Surface category (see Section 4.6)
- Integration with `impact`: `ChangedFile` gets an `api_routes: Vec<ApiRoute>` field; routes are shown in JSON reports

**Edge cases:**

- Routes defined in configuration files (YAML, TOML): not supported in this phase; documented as a known gap
- Dynamic route construction (e.g., `app.use(dynamicPrefix, router)`): mark as `is_dynamic = 1` with `route_confidence = 0.5`
- Route groups/prefixes: concatenate `mount_prefix` with `path_pattern`
- Axum routing via `Router::new().route()`: detect `.route()` call chains; mark as `route_source = 'ROUTER_CHAIN'`
- Multiple frameworks in one repo: detect all, tag each route with its framework

**Acceptance criteria:**

- `changeguard index` populates `api_routes` for Actix, Axum, Rocket, Express, FastAPI, and Flask projects
- `changeguard impact` shows API routes in the JSON report when a route handler changes
- Axum `Router::new().route()` patterns are detected with `route_source = 'ROUTER_CHAIN'`

**Verification gate:**

- Fixture test: Rust Actix project with `#[get("/users")]` extracts route and handler
- Fixture test: Axum project with `Router::new().route("/", get(handler))` extracts route and handler
- Fixture test: Express project with `app.get("/api/users", getUsers)` extracts route and handler
- Impact test: changing a route handler produces structured risk reason with route details

---

#### Track E2-3: Data Model and Entity Extraction

**Goal:** Identify data models, structs, and database schema definitions. Track how data flows through the system by mapping model definitions to their usage sites.

**Deliverables:**

- Data model detection heuristics:
  - Rust: structs with `#[derive(Serialize, Deserialize)]` or `#[derive(sqlx::FromRow)]`; structs in `models/`, `entities/`, or `schema/` directories
  - TypeScript: interfaces/types in `models/`, `types/`, `schemas/` directories; classes extending `Model` or decorated with `@Entity`
  - Python: classes in `models.py` files; classes inheriting from `Base`, `BaseModel`, or `db.Model`
- `data_models` table (Migration M16):

```sql
CREATE TABLE data_models (
    id INTEGER PRIMARY KEY,
    model_name TEXT NOT NULL,
    model_file_id INTEGER NOT NULL REFERENCES project_files(id),
    language TEXT NOT NULL,
    model_kind TEXT NOT NULL DEFAULT 'STRUCT',  -- STRUCT, INTERFACE, CLASS, SCHEMA, GENERATED
    confidence REAL NOT NULL DEFAULT 1.0,
    evidence TEXT,                              -- e.g., "#[derive(Serialize, Deserialize)]"
    last_indexed_at TEXT NOT NULL
);

CREATE INDEX idx_data_models_file ON data_models(model_file_id);
CREATE INDEX idx_data_models_name ON data_models(model_name);
```

- Integration with `structural_edges`: data models that appear as parameter/return types in API routes are linked
- Integration with risk scoring: changed data models receive risk in the Data Contract category (see Section 4.6)

**Edge cases:**

- Anonymous/inline types: skip
- Generated models (e.g., protobuf, OpenAPI): mark as `model_kind = 'GENERATED'` with `confidence = 0.5`
- Very large model files: extract model names only, skip field extraction in this phase

**Acceptance criteria:**

- `changeguard index` populates `data_models` for Rust, TypeScript, and Python
- Changed data models receive elevated risk in the Data Contract category
- Data models are shown in JSON impact reports with confidence and evidence

**Verification gate:**

- Fixture test: Rust struct with `#[derive(Serialize, Deserialize)]` is identified as a data model with evidence
- Fixture test: Python `BaseModel` subclass is identified as a data model

---

#### Track E2-4: Critical Path Analysis

**Goal:** Use the structural call graph and route map to identify "hot" functions that, if broken, would affect the most entry points.

**Current state:** Hotspots combine change frequency and complexity. This track adds *centrality* — how many entry points depend on this function.

**Deliverables:**

- Centrality computation: for each symbol, count the number of entry points (from E1-4) that can reach it via `structural_edges`
- `symbol_centrality` table (Migration M16):

```sql
CREATE TABLE symbol_centrality (
    id INTEGER PRIMARY KEY,
    symbol_id INTEGER NOT NULL REFERENCES project_symbols(id),
    file_id INTEGER NOT NULL REFERENCES project_files(id),
    entrypoints_reachable INTEGER NOT NULL DEFAULT 0,
    betweenness REAL DEFAULT 0.0,
    last_computed_at TEXT NOT NULL,
    UNIQUE(symbol_id)
);

CREATE INDEX idx_symbol_centrality_reachable ON symbol_centrality(entrypoints_reachable);
```

- **Centrality is computed during `changeguard index --analyze-graph`, not during ordinary `impact`.** This keeps `impact` fast. `impact` reads pre-computed centrality values.
- Integration with `hotspots`: add `centrality` as an optional column in the hotspot output (when `structural_edges` data exists)
- Integration with `impact`: symbols with `entrypoints_reachable > 5` receive risk in the Historical Hotspot category (see Section 4.6)

**Edge cases:**

- No call graph data: skip centrality computation, do not add risk weight. Show "Centrality: N/A" in hotspots.
- Cycles in call graph: use BFS with visited set, do not infinite-loop
- Very deep call chains: cap BFS depth at 20 hops

**Acceptance criteria:**

- `changeguard index --analyze-graph` populates `symbol_centrality` when `structural_edges` has data
- `changeguard hotspots --json` includes centrality data when available
- `impact` reads pre-computed centrality but does not compute it

**Verification gate:**

- Fixture test: a function called by 5 route handlers gets centrality = 5
- Hotspot test: centrality column appears in output when structural edges exist
- Performance test: centrality computation on a 1000-symbol fixture completes in < 2 seconds

---

### Phase E3: Observability Wiring

**Intent:** Treat observability (logging, error handling, tracing) as a first-class architectural citizen.

#### Track E3-1: Logging and Event Pattern Detection

**Goal:** Detect logging statements and their patterns in source code.

**Deliverables:**

- Logging pattern extraction via tree-sitter queries:
  - Rust: `log::info!()`, `log::warn!()`, `log::error!()`, `tracing::info!()`, `tracing::warn!()`, `tracing::error!()`, `println!()`, `eprintln!()`
  - TypeScript: `console.log()`, `console.warn()`, `console.error()`, `logger.info()`, `logger.warn()`, `logger.error()`, `winston.log()`
  - Python: `logging.info()`, `logging.warning()`, `logging.error()`, `print()`, `logger.info()`
- `observability_patterns` table (Migration M17):

```sql
CREATE TABLE observability_patterns (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES project_files(id),
    line_start INTEGER,
    pattern_kind TEXT NOT NULL DEFAULT 'LOG',  -- LOG, ERROR_HANDLE, TRACE
    level TEXT,                            -- debug, info, warn, error
    framework TEXT,                        -- log, tracing, console, logging, etc.
    confidence REAL NOT NULL DEFAULT 1.0,
    evidence TEXT,
    in_test INTEGER DEFAULT 0,            -- 1 if in a test file
    last_indexed_at TEXT NOT NULL
);

CREATE INDEX idx_obs_patterns_file ON observability_patterns(file_id);
CREATE INDEX idx_obs_patterns_kind ON observability_patterns(pattern_kind);
```

- **Logging coverage deltas**: during `impact`, compare the changed file's current observability patterns against the HEAD version's patterns (using `gix` diff). A decrease in logging statements produces an analysis warning with evidence. This is a comparison against git HEAD, not against a stored snapshot, keeping it simple and always current.
- Integration with `impact`: when a changed file's logging statements decrease between working tree and HEAD, add "Logging coverage reduced in X: N statements removed" to `analysis_warnings`

**Edge cases:**

- Commented-out logging: do not count as logging (tree-sitter will not match commented nodes)
- Macro-generated logging (e.g., `#[instrument]`): detect the attribute, count as TRACE
- Logging in test files: mark with `in_test = 1`, exclude from coverage metrics

**Acceptance criteria:**

- `changeguard index` populates `observability_patterns` for supported languages
- `changeguard impact` warns when logging coverage decreases in a changed file (comparison against HEAD)
- Test files' logging patterns are excluded from coverage metrics

**Verification gate:**

- Fixture test: Rust file with `tracing::info!()` and `tracing::error!()` is correctly cataloged
- Impact test: removing a logging statement and running impact produces an analysis warning

---

#### Track E3-2: Error Handling Pattern Detection

**Goal:** Map error handling patterns. Flag changes that reduce error handling.

**Deliverables:**

- Error handling pattern extraction:
  - Rust: `match` on `Result`/`Option` (syntactic evidence only — we detect the `match` pattern, not whether it's on a `Result`), `.unwrap()`, `.expect()`, `?` operator, `anyhow!`, `thiserror`
  - TypeScript: `try/catch/finally`, `.catch()`, `Promise.reject`, `throw`
  - Python: `try/except/finally`, `raise`, `assert`
- Extend `observability_patterns` with `pattern_kind = 'ERROR_HANDLE'`
- Integration with `impact`: when error handling patterns decrease between working tree and HEAD, produce "Error handling coverage reduced in X" warning
- Integration with risk scoring: changes to error handling in `Infrastructure` directories receive risk in the Runtime/Config category (see Section 4.6)
- **Label error handling as "syntactic evidence"**: tree-sitter can detect `.unwrap()`, `.expect()`, `?`, and `match`, but cannot always determine whether a `match` is on `Result` or `Option` without type information. Mark these as `evidence = "syntactic: match expression"` rather than confirmed.

**Edge cases:**

- `.unwrap()` in test files: not flagged (test code commonly uses unwrap)
- `.expect("message")` vs `.unwrap()`: `expect` is slightly better but still flagged in production code with `confidence = 0.9`
- Error handling in generated code: skip

**Acceptance criteria:**

- `changeguard index` populates `observability_patterns` with error handling entries
- `changeguard impact` warns when error handling is reduced
- Test files are exempt from unwrap warnings

**Verification gate:**

- Fixture test: Rust file with `match` and `.unwrap()` patterns is correctly cataloged with syntactic evidence labels
- Impact test: replacing `match` with `.unwrap()` in production code produces a warning

---

#### Track E3-3: Telemetry and Trace Wiring

**Goal:** Detect OpenTelemetry, Prometheus, and custom metrics usage.

**Deliverables:**

- Telemetry pattern extraction:
  - Rust: `#[instrument]`, `opentelemetry::`, `prometheus::`, `metrics::`
  - TypeScript: `@Trace()`, `opentelemetry`, `prom-client`, `metrics`
  - Python: `@tracer.start_as_current_span`, `opentelemetry`, `prometheus_client`, `metrics`
- Extend `observability_patterns` with `pattern_kind = 'TRACE'`
- Integration with `impact`: changes that remove telemetry instrumentation produce "Telemetry coverage reduced" warning (comparison against HEAD)
- Integration with LSP: observability pattern count shown in hover

**Edge cases:**

- Custom telemetry wrappers: detect common patterns (`telemetry.`, `monitoring.`) as `framework = 'custom'` with `confidence = 0.7`
- Telemetry in libraries: do not flag changes to library telemetry
- Missing telemetry in new code: optional `--telemetry-coverage` flag on `impact` to surface files that *should* have telemetry but don't

**Acceptance criteria:**

- `changeguard index` populates `observability_patterns` with trace entries
- `changeguard impact --telemetry-coverage` warns about removed telemetry
- LSP hover shows observability pattern count for the file

**Verification gate:**

- Fixture test: Rust file with `#[instrument]` is correctly cataloged
- Impact test: removing `#[instrument]` produces a warning

---

### Phase E4: Safety Context

**Intent:** Close the loop between "what changed" and "how do we verify it."

#### Track E4-3: Environment Variable Schema Extraction

**Goal:** Extract environment variable declarations and references separately. Flag changes that introduce new env var dependencies that aren't declared anywhere.

**Current state:** `src/index/runtime_usage.rs` extracts env var references from source code but doesn't distinguish declarations from usage, and doesn't compare against config files.

**Deliverables:**

- Two separate tables for declarations vs. references:

```sql
CREATE TABLE env_declarations (
    id INTEGER PRIMARY KEY,
    var_name TEXT NOT NULL,
    source_file_id INTEGER NOT NULL REFERENCES project_files(id),
    source_kind TEXT NOT NULL,           -- DOTENV_EXAMPLE, CONFIG, DOCS
    required INTEGER DEFAULT 0,
    default_value_redacted TEXT,         -- HAS_DEFAULT, EMPTY_DEFAULT, PLACEHOLDER_DEFAULT, POSSIBLE_SECRET_REDACTED
    description TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_indexed_at TEXT NOT NULL,
    UNIQUE(var_name, source_file_id, source_kind)
);

CREATE TABLE env_references (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES project_files(id),
    symbol_id INTEGER REFERENCES project_symbols(id),
    var_name TEXT NOT NULL,
    reference_kind TEXT NOT NULL,        -- READ, WRITE, DEFAULTED, DYNAMIC
    line_start INTEGER,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_indexed_at TEXT NOT NULL
);

CREATE INDEX idx_env_decls_var ON env_declarations(var_name);
CREATE INDEX idx_env_refs_var ON env_references(var_name);
CREATE INDEX idx_env_refs_file ON env_references(file_id);
```

- Config file parsing:
  - `.env.example`, `.env.template`: extract variable names (never read actual `.env` files)
  - `config.toml`, `config.json`: extract keys and types
  - Source code env var references (already extracted by `runtime_usage`)
- **Never store raw default values from `.env.example`** — store only `HAS_DEFAULT`, `EMPTY_DEFAULT`, `PLACEHOLDER_DEFAULT`, or `POSSIBLE_SECRET_REDACTED` to prevent accidental secret exposure
- Integration with `impact`: env var references that have no corresponding declaration produce "Undeclared env var dependency: X" warning
- Integration with `verify`: suggest running with `--check-env` to validate that all required env vars are set (future phase)

**Edge cases:**

- `.env` files (not `.env.example`): **never read or parse** actual `.env` files (security)
- Dynamic env var names (`process.env[DYNAMIC_KEY]`): mark as `reference_kind = 'DYNAMIC'` with `confidence = 0.5`
- Same env var referenced from multiple files: one `env_declaration` per source file, one `env_reference` per reference

**Acceptance criteria:**

- `changeguard index` populates `env_declarations` and `env_references`
- `changeguard impact` warns about undeclared env var dependencies
- Actual `.env` files are never read
- Raw default values are never stored; only redacted categories are stored

**Verification gate:**

- Fixture test: `.env.example` with `DATABASE_URL=postgres://...` produces `HAS_DEFAULT` not the actual URL
- Impact test: adding `std::env::var("NEW_VAR")` to a source file produces an "Undeclared env var dependency" warning

---

#### Track E4-4: Runtime Usage in Risk Scoring

**Goal:** Wire the already-extracted `runtime_usage` (env vars, config keys) into the risk scoring and verification prediction systems.

**Current state:** `ChangedFile.runtime_usage` is populated during `impact` but has zero effect on risk scoring or prediction.

**Deliverables:**

- Extend `analyze_risk()` in `src/impact/analysis.rs` with category-capped scoring (see Section 4.6):
  - Files with new env var references not in `env_declarations`: Runtime/Config category risk
  - Files that change env var references: Runtime/Config category risk
  - Files that change config key references: Runtime/Config category risk
- Extend verification prediction in `src/verify/predict.rs`:
  - When a changed file introduces new env var dependencies, add a prediction reason with evidence
  - When a changed file removes env var references, add a warning
- Add `runtime_usage` to `impact` JSON report output (currently collected but not included in the serialized packet)

**Edge cases:**

- `runtime_usage` extraction failure: degrade gracefully, skip risk weight for that file
- Very common env vars (`PATH`, `HOME`): skip in risk scoring (too common to be meaningful)
- Config keys that are framework conventions (`server.port`, `logging.level`): reduced weight

**Acceptance criteria:**

- `changeguard impact` gives category-capped risk to files with new env var dependencies
- `changeguard verify` includes env-var-based predictions in verification plans
- `impact` JSON report includes `runtime_usage` data

**Verification gate:**

- Risk scoring test: file with new env var reference receives Runtime/Config category risk
- Verify test: file with new env var dependency gets a prediction reason
- JSON report test: `runtime_usage` appears in the serialized output

---

#### Track E4-1: Test-to-Symbol Mapping

**Goal:** Map test functions to the symbols they test. Use this mapping to improve verification prediction.

**Current state:** Verification prediction uses structural imports and temporal coupling. It does not know which test function tests which symbol.

**Deliverables:**

- Test detection and mapping:
  - Rust: `#[test]` and `#[tokio::test]` functions → map to the module/function they import
  - TypeScript: `describe()`, `it()`, `test()` blocks → map to the imported module
  - Python: `def test_*()` functions → map to imported modules
- `test_mapping` table (Migration M18):

```sql
CREATE TABLE test_mapping (
    id INTEGER PRIMARY KEY,
    test_symbol_id INTEGER NOT NULL REFERENCES project_symbols(id),
    test_file_id INTEGER NOT NULL REFERENCES project_files(id),
    tested_symbol_id INTEGER REFERENCES project_symbols(id),
    tested_file_id INTEGER REFERENCES project_files(id),
    confidence REAL NOT NULL DEFAULT 1.0,
    mapping_kind TEXT NOT NULL DEFAULT 'IMPORT',  -- IMPORT, NAMING_CONVENTION, COVERAGE_DATA
    evidence TEXT,                              -- e.g., "imports foo::bar directly"
    last_indexed_at TEXT NOT NULL,
    UNIQUE(test_symbol_id, tested_symbol_id)
);

CREATE INDEX idx_test_mapping_tested ON test_mapping(tested_symbol_id);
CREATE INDEX idx_test_mapping_test ON test_mapping(test_symbol_id);
```

- Integration with `verify`: add test-mapping-based prediction as Priority 1 (before temporal and structural prediction)
- Integration with `impact`: show "Tests covering this change: X" in the JSON report

**Edge cases:**

- Integration tests that test multiple modules: create multiple mappings with reduced `confidence`
- Tests with no clear import relationship to tested code: mark as `mapping_kind = 'NAMING_CONVENTION'` with `confidence = 0.5`
- No test files in the repo: skip, no prediction from test mapping
- Test files in a separate directory (`tests/` at repo root in Rust): use module path resolution to map

**Acceptance criteria:**

- `changeguard index` populates `test_mapping` for supported test frameworks
- `changeguard verify` includes test-mapping-based predictions in the verification plan
- `changeguard impact` shows test coverage information when available

**Verification gate:**

- Fixture test: Rust test `test_foo()` that imports `foo::bar()` maps to `bar` with `mapping_kind = 'IMPORT'`
- Verify test: changing `bar()` predicts running `test_foo`
- Impact test: changing `bar()` shows "Tests covering: test_foo" in the report

---

#### Track E4-2: CI/CD Workflow Awareness

**Goal:** Parse CI/CD configuration files to understand what gates exist and flag changes that could affect pipeline behavior.

**Deliverables:**

- CI config parsing:
  - GitHub Actions: `.github/workflows/*.yml` → extract jobs, triggers, steps
  - GitLab CI: `.gitlab-ci.yml` → extract stages, jobs, scripts
  - CircleCI: `.circleci/config.yml` → extract jobs, workflows
  - Makefile: `Makefile` → extract targets and dependencies
- `ci_gates` table (Migration M18):

```sql
CREATE TABLE ci_gates (
    id INTEGER PRIMARY KEY,
    ci_file_id INTEGER NOT NULL REFERENCES project_files(id),
    platform TEXT NOT NULL,              -- github_actions, gitlab_ci, circleci, makefile
    job_name TEXT NOT NULL,
    trigger TEXT,                         -- push, pull_request, schedule, etc.
    steps JSON,                         -- [{name, run}]
    last_indexed_at TEXT NOT NULL
);

CREATE INDEX idx_ci_gates_file ON ci_gates(ci_file_id);
CREATE INDEX idx_ci_gates_platform ON ci_gates(platform);
```

- Integration with `impact`: changes to CI files produce structured risk reasons in the Verification Gap category
- Integration with `verify`: suggest running CI-matching verification commands
- **Platform awareness**: only suggest commands that exist on the current platform. `make test` is only suggested if `make` is in PATH. Use confidence levels:
  - `HIGH`: command exists in PATH and config declares it
  - `MEDIUM`: config declares it but executable not found in PATH
  - `LOW`: inferred from ecosystem (e.g., Rust project → `cargo test`)

**Edge cases:**

- No CI config: skip, no warnings
- Malformed YAML: extract what's possible, mark as `parse_status = 'PARSE_FAILED'`
- CI config that references external secrets: do not extract secret values, note that the job uses secrets
- Multiple CI platforms in one repo: index all

**Acceptance criteria:**

- `changeguard index` populates `ci_gates` when CI config files exist
- `changeguard impact` flags CI config changes with elevated risk
- `changeguard verify` suggests CI-matching verification commands with confidence levels

**Verification gate:**

- Fixture test: `.github/workflows/ci.yml` with `test` job is parsed correctly
- Impact test: changing `.github/workflows/ci.yml` produces a CI-related risk reason
- Verify test: `verify` suggests `cargo test` for Rust projects with HIGH confidence

---

## 4. Technical Architecture Updates

### 4.1 Database Schema Evolution

All expansion migrations are additive and backward-compatible:

| Migration | Tables Added | Notes |
|-----------|--------------|-------|
| M15 | `project_files`, `project_symbols`, `project_docs`, `project_topology`, `index_metadata` | Foundation F0/F1 + Phase E1 |
| M16 | `structural_edges`, `api_routes`, `data_models`, `symbol_centrality` | Phase E2 |
| M17 | `observability_patterns` | Phase E3 |
| M18 | `test_mapping`, `ci_gates`, `env_declarations`, `env_references` | Phase E4 |

The existing `symbols`, `snapshots`, `changed_files`, `transactions`, `ledger_entries` tables are unchanged. New tables use integer foreign keys (`file_id`, `symbol_id`) referencing `project_files` and `project_symbols`, not `file_path` / `symbol_name` strings. Unresolved references fall back to text fields.

### 4.2 Risk Scoring Model

The current risk scoring is purely additive, which can produce meaningless numbers. The expansion switches to **category-capped scoring**:

```text
Base score: 0
Category maxes:
- API Surface (entry points, routes, public API):     max 35 points
- Data Contract (data models, serialization):          max 35 points
- Runtime/Config (env vars, config keys, CI changes):  max 25 points
- Verification Gap (missing tests, CI):                max 30 points
- Historical Hotspot (frequency, complexity, centrality): max 30 points
- Observability Reduction (removed logging/error handling): max 25 points

Final score = min(100, sum of category scores)
Risk band: >60 = High, >25 = Medium, else Low
```

Each risk reason includes structured evidence:

```json
{
  "kind": "API_ROUTE",
  "weight": 30,
  "confidence": 0.95,
  "evidence": "GET /users handled by list_users in src/api.rs:42"
}
```

### 4.3 New CLI Commands and Flags

| Command | Phase | Notes |
|---------|-------|-------|
| `changeguard index [--incremental] [--analyze-graph]` | E1/F2 | Full or incremental project indexing; `--analyze-graph` computes centrality |
| `changeguard index --check` | F1 | Show indexing status without re-indexing |
| `changeguard hotspots --centrality` | E2 | Include centrality data in hotspot output |
| `changeguard impact --telemetry-coverage` | E3 | Surface files missing telemetry |
| `changeguard ledger track <entity> <action>` | E4 | Manually attach token provenance (fills L5-1 gap) |

Existing commands receive additive flags only. No existing flags change behavior.

### 4.4 Dependency Additions

| Crate | Version | Phase | Justification |
|-------|---------|-------|---------------|
| `pulldown-cmark` | 0.13.3 | E1-2 | Markdown parsing for README ingestion; pure Rust, no native deps |

All other Phase E functionality is built on existing dependencies. CI config parsing will use a shallow line-oriented scanner rather than adding a YAML dependency — the existing `serde` + `toml` crates can handle simple YAML structures via text patterns for the CI fields we need (job names, triggers, run commands).

**Explicitly deferred:** Semantic indexing via embeddings (`candle`, `ort`, or similar). The four-layer model can be fully implemented with deterministic tree-sitter queries and graph algorithms. Semantic indexing would add significant native dependency risk and is not required for the stated capabilities.

### 4.5 Impact Packet Schema Extensions

The `ImpactPacket` struct gains these additive fields (all optional, defaulting to empty/None):

```rust
pub struct ImpactPacket {
    // ... existing fields unchanged ...
    pub api_routes: Vec<ApiRoute>,                  // E2-2
    pub data_models: Vec<DataModel>,               // E2-3
    pub observability_warnings: Vec<ObservabilityWarning>, // E3: logging/error/telemetry changes
    pub env_var_deps: Vec<EnvVarDep>,              // E4-3: undeclared env var dependencies
    pub test_coverage: Vec<TestCoverage>,          // E4-1: tests covering changed symbols
}
```

Each `ObservabilityWarning` and `EnvVarDep` includes a `confidence` field and `evidence` string.

All new fields have `#[serde(default)]` so older JSON consumers are not broken.

### 4.6 LSP Daemon Extensions

| Feature | Phase | Notes |
|---------|-------|-------|
| Symbol-level diagnostics | E1-4 | Replace position (0,0) with actual symbol line ranges from `project_symbols` |
| Route annotations in CodeLens | E2-2 | Show "GET /users" above handler functions |
| Observability pattern count in Hover | E3-3 | Show "3 log statements, 1 trace" on hover |
| Entry point markers in CodeLens | E1-4 | Show "Entry Point" / "Handler" / "Test" / "Public API" |

**Position encoding rule:** Store byte offsets and 1-based display lines in SQLite. Convert to LSP zero-based UTF-16 positions at the LSP boundary. This prevents Unicode-related diagnostic placement bugs.

All LSP extensions are additive. Existing diagnostics, hover, and CodeLens behavior is preserved when `project_symbols` data is unavailable.

### 4.7 Windows Path Normalization

Because ChangeGuard is Windows-first, normalize paths aggressively throughout the expansion:

- Store repo-relative paths using forward slashes in all SQLite tables
- Preserve original OS path only for display
- Case-fold only where the filesystem is case-insensitive (detect via `.GIT` existence)
- Never store absolute user paths in reports unless explicitly requested

This matters for federation, test mapping, and cross-repo impact.

---

## 5. Priority Gaps to Close Before Expansion

These are existing gaps that must be closed before or alongside the expansion phases. They prevent the expansion from building on unstable foundations.

### 5.1 Ledger Verification Gate (Pre-E1)

**Gap:** Categories like ARCHITECTURE, FEATURE, BUGFIX, and INFRA require `verification_status` and `verification_basis` at commit time, but `commit_change` does not enforce this.

**Fix:** In `src/ledger/transaction.rs`, add a check in `commit_change` that rejects commits for enforcement-required categories when `verification_status` is `None`. Gate behind `config.ledger.verify_to_commit`. Add `--force` flag to override. This is a small, focused change.

### 5.2 Hotspot Complexity Fallback (Pre-E1)

**Gap:** `hotspots` returns complexity 0 for all files when no prior `impact` run has populated the `symbols` table.

**Fix:** When the `symbols` table is empty, query the `project_symbols` table (from E1-1) as a fallback. The fallback must gracefully handle the case where `project_symbols` doesn't exist yet (pre-Migration-M15 databases).

### 5.3 Federated Dependency Matching (E1 or E2)

**Gap:** `discover_dependencies_in_current_repo` uses `file_content.contains(symbol_to_find)` which produces false positives.

**Fix:** Replace substring matching with word-boundary matching using `Regex::new(r"\b{symbol}\b")` or the already-extracted import data from `src/index/references.rs`.

### 5.4 Token Budget Consistency (Pre-E1)

**Gap:** The `execute_ask` function hardcodes `truncate_for_context(409_600)` while `config.gemini.context_window` defaults to 128,000 but is never used.

**Fix:** Derive the truncation budget from `config.gemini.context_window` (80% of context_window × 4 chars/token). Add a minimum floor of 32,768 chars to prevent zero-truncation on misconfiguration.

### 5.5 LSP Position Fix (E1-4)

**Gap:** All LSP diagnostics and CodeLens use position (0,0) instead of actual symbol positions.

**Fix:** When `project_symbols` has `line_start` and `line_end` data, use those for diagnostic and CodeLens positions (with LSP position encoding: 0-based lines, 0-based columns). Fall back to (0,0) when no symbol data is available.

---

## 6. Implementation Sequence

The phases must be implemented in order because each builds on the previous. Within each phase, some tracks can be partially parallelized.

```
F0 → F1 → Pre-E1 fixes (5.1, 5.2, 5.4) → E1-1 → E1-1b (complexity + hotspot fallback) → E1-3 → E1-4 → E1-2
E2-1 → E2-2 → E2-3 → E2-4 (offline, during index --analyze-graph)
E3-1 → E3-2 → E3-3
E4-3 → E4-4 → E4-1 → E4-2
```

### Milestone Organization

**Milestone M0 — Foundation (F0, F1, Pre-E1 fixes)**
Tracks F0, F1, E0-1, E0-2, E0-3, E0-4. Must be complete and tested before any E-track.

**Milestone M1 — Structural Bearings (E1)**
Tracks E1-1, E1-1b (complexity + hotspot fallback), E1-3, E1-4, E1-2.

**Milestone M2 — Behavioral Mapping (E2)**
Tracks E2-1, E2-2, E2-3, E2-4. Plus Pre-E1 fix 5.3 (federated matching).

**Milestone M3 — Observability Wiring (E3)**
Tracks E3-1, E3-2, E3-3. Plus Pre-E1 fix 5.5 (LSP positions).

**Milestone M4 — Safety Context (E4)**
Tracks E4-3, E4-4, E4-1, E4-2. (Env handling before test mapping per review.)

Each milestone should be fully tested and production-stable before the next begins. Each track within a milestone should follow the conductor workflow: plan → push plan → implement → review → merge.

---

## 7. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tree-sitter queries produce false positives for route/model detection | Medium | Medium | Label low-confidence matches with `confidence < 1.0` and `evidence`. Never present speculation as fact. |
| Full-project indexing is too slow on large repos | Medium | High | Incremental indexing (content-hash + parser-version). Batch inserts. Progress bar. Cap at 10K files per language. |
| Call graph exceeds memory on monorepos | Low | High | Cap at 50K edges per file. Stream edges to SQLite during extraction. Mark as `resolution_status = 'CAPPED'`. |
| New migrations break existing `ledger.db` | Low | Critical | All migrations are additive. Test with real `ledger.db` from production. |
| `pulldown-cmark` adds native dependency risk | Low | Low | Pure Rust crate, no native deps. Version 0.13.3 is current stable. |
| LSP position fix requires symbol data that doesn't exist yet | Medium | Low | Fall back to (0,0) when `project_symbols` has no data for a file. |
| CI config parsing fails on exotic YAML | Medium | Low | Use shallow line-oriented scanner, not full YAML parser. Extract what's possible. Mark failures as `PARSE_FAILED`. Never crash. |
| Verification gate enforcement breaks existing workflows | Medium | High | Default `enforcement_enabled = false`. Add `--force` override. Log a warning when enforcement would have blocked. |
| Risk scoring becomes meaningless with too many additive weights | Medium | High | Use category-capped scoring (Section 4.2). Each category has a max. Total is capped at 100. |
| Stale index due to mtime/checkout issues on Windows | Medium | Medium | Three-part invalidation: content_hash + git_blob_oid + parser_version. Never rely on mtime alone. |
| Duplicate symbol names across modules cause false matches | High | High | Use `qualified_name` (e.g., `crate::module::function`) and integer `symbol_id` references, not bare `symbol_name` strings. |

---

## 8. Testing Strategy

### Per-Track Verification

Each track must include:
1. **Unit tests** for new extraction logic (tree-sitter queries, pattern matchers, heuristics)
2. **Integration tests** with fixture repos containing real patterns (Rust, TypeScript, Python)
3. **Regression tests** ensuring existing `impact`, `hotspots`, `verify`, and `ledger` commands are unaffected

### Cross-Phase Integration Tests

After each milestone:
1. Run the full existing test suite (`cargo test`)
2. Run `changeguard index` on the ChangeGuard repo itself
3. Run `changeguard impact` and verify that new fields appear in JSON output alongside existing fields
4. Run `changeguard hotspots` and verify that centrality data appears when available
5. Run `changeguard verify` and verify that test-mapping predictions appear alongside existing predictions
6. Verify LSP daemon still starts and serves diagnostics, hover, and CodeLens

### Performance Gates

| Command | Maximum Time | Repo Size |
|---------|-------------|-----------|
| `changeguard index` (full) | 30 seconds | 2000 source files |
| `changeguard index` (incremental) | 5 seconds | 50 changed files |
| `changeguard index --analyze-graph` | 60 seconds | 2000 source files |
| `changeguard impact` | 5 seconds | 200 changed files |
| `changeguard hotspots` | 10 seconds | 10,000 commits |
| `changeguard verify` | 2 seconds (planning) | 200 changed files |

---

## 9. Final Implementation Warning

The most likely way to fail this expansion is to build intelligent features before the data foundation is solid.

The `project_files` and `project_symbols` tables (F0 + E1-1) must exist and be populated before hotspots can fall back to them, before call graphs can reference them, before entry points can be labeled, and before test mapping can resolve symbol names.

**Reliability comes first. Sophistication comes second.**

Every track must degrade gracefully. If `project_symbols` is empty, `hotspots` must still work (with 0 complexity). If `structural_edges` is empty, `verify` must still predict using imports and temporal coupling. If `ci_gates` is empty, `impact` must still score risk without CI awareness. Missing data is a visible warning, never a crash.

Every extracted fact must carry a `confidence` score and `evidence` string. Low-confidence facts are labeled as such, never presented as confirmed. This is the difference between a useful intelligence system and a misleading one.
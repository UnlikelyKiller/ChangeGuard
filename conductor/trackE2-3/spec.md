# Specification: Track E2-3 - Data Model and Entity Extraction

## 1. Objective
Identify data models, structs, and database schema definitions in source code. Distinguish data models from regular structs/classes by detecting serialization attributes, ORM annotations, and naming conventions. Store results in a new `data_models` table (Migration M16) and integrate with risk scoring (+40 for changed data models, since they represent contracts). This track runs in parallel with E2-2 after E2-1 lands.

## 2. Deliverables

### 2.1 Data Model Detection Heuristics
- **Target files**: `src/index/languages/rust.rs`, `src/index/languages/typescript.rs`, `src/index/languages/python.rs`
- **Details**: Add data model detection to each language module. A symbol is classified as a data model when it meets any of these criteria:
  - **Rust**:
    - Structs with `#[derive(Serialize, Deserialize)]` or `#[derive(serde::Serialize, serde::Deserialize)]`
    - Structs with `#[derive(sqlx::FromRow)]`, `#[derive(diesel::Queryable)]`, `#[derive(Debug, Clone, Serialize)]` (any derive that includes `Serialize` or `Deserialize`)
    - Structs in `models/`, `entities/`, `schema/`, or `domain/` directories (naming convention)
    - Structs with `#[serde(rename_all = "...")]` attribute (serialization hint)
  - **TypeScript**:
    - Interfaces/types in `models/`, `types/`, `schemas/`, or `interfaces/` directories
    - Classes extending `Model` (e.g., Sequelize, Objection.js)
    - Classes/interfaces decorated with `@Entity` (TypeORM)
    - Types/interfaces with `z.object()` or `z.string()` Zod schema definitions nearby
  - **Python**:
    - Classes in `models.py` files or `models/` packages
    - Classes inheriting from `BaseModel` (Pydantic), `Base` (SQLAlchemy declarative_base), `db.Model` (Flask-SQLAlchemy), `Model` (Django)
    - Classes decorated with `@dataclass` in `models/` directories
    - Classes inheriting from `pydantic.BaseModel`

### 2.2 Data Models Table (Migration M16)
- **Target file**: `src/state/migrations.rs`
- **Details**: Add to Migration M16 (shared with E2-1 and E2-2) creating the `data_models` table:
  ```sql
  CREATE TABLE IF NOT EXISTS data_models (
      id              INTEGER PRIMARY KEY AUTOINCREMENT,
      model_name      TEXT NOT NULL,
      model_file_id   INTEGER NOT NULL REFERENCES project_files(id),
      language        TEXT NOT NULL,
      model_kind      TEXT NOT NULL DEFAULT 'STRUCT',
      confidence      REAL NOT NULL DEFAULT 1.0,
      evidence        TEXT,
      fields          TEXT,  -- JSON, reserved for future use
      last_indexed_at TEXT NOT NULL,
      FOREIGN KEY (model_file_id) REFERENCES project_files(id)
  );
  CREATE INDEX IF NOT EXISTS idx_data_models_name
      ON data_models(model_name);
  CREATE INDEX IF NOT EXISTS idx_data_models_file
      ON data_models(model_file_id);
  ```
- The `model_kind` enum: `STRUCT` (Rust structs, Python classes), `INTERFACE` (TypeScript interfaces), `CLASS` (Python classes with ORM base), `SCHEMA` (Zod schemas, Serde-with-attributes), `GENERATED` (auto-generated models from protobuf, OpenAPI, GraphQL schemas). Stored as TEXT.
- The `confidence` column stores a value between 0.0 and 1.0 indicating detection confidence. Models identified by serialization derives or ORM base classes default to 1.0. Models identified by directory naming convention default to 0.7. Generated models default to 0.6.
- The `evidence` column stores an optional JSON string describing what evidence led to classification (e.g., `"derive: Serialize, Deserialize"`, `"dir: models/"`, `"base: BaseModel"`).
- The `fields` column is `TEXT` (JSON) reserved for future field-level extraction. In this phase, it is `NULL`.

### 2.3 Data Model Extraction Module
- **Target file**: New `src/index/data_models.rs`
- **Details**: Implement `DataModelExtractor` that:
  1. Queries `project_symbols` for struct, class, and interface symbols, and `project_files` for file IDs.
  2. Applies language-specific heuristics to classify which symbols are data models.
  3. For Rust: scans derive attributes and directory paths. A struct with `Serialize` or `Deserialize` in its derives is a data model (`confidence = 1.0`, `evidence = "derive: Serialize, Deserialize"`). A struct in a `models/` directory is a data model with reduced confidence (`confidence = 0.7`, `evidence = "dir: models/"`).
  4. For TypeScript: scans for `@Entity` decorators, `extends Model`, and directory paths. An interface in `models/` is a data model.
  5. For Python: scans for base classes (`BaseModel`, `Base`, `db.Model`), file paths (`models.py`, `models/` directory), and decorators (`@dataclass` in model directories).
  6. Marks generated models (e.g., in `Generated` directories from E1-3 topology) with `model_kind = 'GENERATED'`, `confidence = 0.6`, and `evidence` noting the generation source.
  7. Populates `model_file_id` by joining detected models to `project_files(id)` instead of storing file paths as text.
  8. Streams extracted models to SQLite.

### 2.4 Impact Integration
- **Target file**: `src/impact/analysis.rs`
- **Details**:
  - When a changed file contains data model symbols, query `data_models` for matching models (joining on `model_file_id`).
  - Changed data models contribute up to 35 points within the Data Contract category (max 35 points). Generated models contribute up to 20 points.
  - Add risk reason: `"Data model: {model_name} ({model_kind})"`.
  - Generated data models (in `Generated` directories, `model_kind = 'GENERATED'`) receive reduced weight (up to 20 points within Data Contract instead of up to 35).
  - Extend `ChangedFile` in `src/impact/packet.rs` with a `data_models: Vec<DataModel>` field (with `#[serde(default)]`).

### 2.5 Structural Edge Integration
- **Target file**: `src/index/call_graph.rs` (from E2-1)
- **Details**: Data models that appear as parameter or return types in API routes (from E2-2) should be linked. When a route handler's signature references a data model (e.g., `Json<UserModel>` in Actix, `response_model=UserModel` in FastAPI), the `structural_edges` table gains an edge from the handler to the model. This is a future enhancement; this track stores the `data_models` data that makes it possible.

### 2.6 Index Command Integration
- **Target file**: Command handler for `changeguard index`
- **Details**: Data model extraction runs after `project_symbols` is populated. Invoked as part of the `changeguard index` pipeline.

## 3. Constraints & Guidelines
- **Deterministic over speculative**: If a struct/class might be a data model but does not meet any clear criterion, do not classify it. A false negative (missing a data model) is better than a false positive (marking an internal struct as a data model).
- **Graceful degradation**: If no data models are detected, the `data_models` table remains empty and `impact` proceeds normally.
- **No performance regression**: Data model detection must add less than 1 second to `changeguard index` for a 2000-file repo.
- **Field extraction scope**: This phase extracts model names, files, kinds, and languages only. Field-level extraction (the `fields` JSON column) is deferred to a future phase. The `fields` column is left as `NULL`.
- **Backward-compatible schema**: The `data_models` table is additive. No existing table is modified.

## 4. Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Anonymous/inline types | Skip. Only named structs, classes, and interfaces are classified. |
| Generated models (protobuf, OpenAPI, GraphQL) | Detect via `Generated` directory classification (from E1-3). Mark with `model_kind = 'GENERATED'`, `confidence = 0.6`, and `evidence` noting the generation source. Apply reduced risk weight (up to 20 points within Data Contract instead of up to 35). |
| Very large model files | Extract model names and kinds only. Skip field extraction. |
| Multiple derive attributes on same struct | Parse all derives. If any include `Serialize` or `Deserialize`, classify as data model. |
| Structs in `models/` directory without Serde | Classify as data model (naming convention) but with `model_kind = 'STRUCT'`. No serialization guarantee. |
| Python `@dataclass` outside `models/` | Do not classify as data model unless in a `models/` directory or inheriting from a model base class. |
| TypeScript `interface` in business logic | Only classify if in `models/`, `types/`, `schemas/`, or `interfaces/` directory. |
| Same model name in multiple files | Store each occurrence as a separate row. The `model_file_id` column disambiguates. |
| Refactored/relocated models | Incremental indexing replaces rows for re-indexed files. |

## 5. Acceptance Criteria

1. `changeguard index` populates `data_models` for Rust structs with `#[derive(Serialize, Deserialize)]`, TypeScript interfaces in `models/` directories, and Python classes inheriting from `BaseModel` or `Base`.
2. Changed data models contribute up to 35 points within the Data Contract category (max 35 points) in `impact`.
3. Generated data models (`model_kind = 'GENERATED'`) contribute up to 20 points within the Data Contract category (reduced).
4. `ChangedFile.data_models` field appears in serialized `ImpactPacket` JSON output.
5. Data model classification is conservative: internal utility structs without serialization attributes are not classified as data models.
6. Repos without data models produce empty `data_models` table and no warnings or errors.

## 6. Verification Gate

- **Fixture test (Rust)**: A Rust struct with `#[derive(Serialize, Deserialize)]` is identified as a data model with `model_kind = 'STRUCT'`.
- **Fixture test (Rust)**: A Rust struct with `#[derive(sqlx::FromRow)]` is identified as a data model.
- **Fixture test (Rust)**: A plain Rust struct without serialization derives in `src/` is NOT classified as a data model.
- **Fixture test (TypeScript)**: A TypeScript interface in `models/user.ts` is identified as a data model with `model_kind = 'INTERFACE'`.
- **Fixture test (Python)**: A Python class inheriting from `BaseModel` is identified as a data model with `model_kind = 'CLASS'`.
- **Fixture test (Python)**: A Python class inheriting from `db.Model` (Flask-SQLAlchemy) is identified as a data model.
- **Impact test**: Changing a data model struct produces risk reason `"Data model: UserModel (STRUCT)"` and up to 35 points within the Data Contract category.
- **Impact test**: Changing a generated model produces up to 20 points within the Data Contract category (reduced).
- **JSON report test**: The `data_models` field appears in the serialized `ImpactPacket` for changed files containing data models.
- **Empty-table test**: With no `data_models` data, `impact` produces output identical to the baseline.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M16 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E2 tables for fixture repos
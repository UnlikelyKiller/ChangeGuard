## Plan: Track E2-3 - Data Model and Entity Extraction

### Phase 1: Database Schema
- [ ] Task 1.1: Add `data_models` table creation to Migration M16 in `src/state/migrations.rs` (shared with E2-1, E2-2). Columns: `id`, `model_name`, `model_file_id` (INTEGER NOT NULL REFERENCES project_files(id)), `language`, `model_kind`, `confidence` (REAL NOT NULL DEFAULT 1.0), `evidence` (TEXT, nullable), `fields` (TEXT/JSON, nullable), `last_indexed_at`. Include indices on `model_name` and `model_file_id`.
- [ ] Task 1.2: Add `data_models` to the `test_all_tables_exist` test in `src/state/migrations.rs`.
- [ ] Task 1.3: Write a new test `test_insert_and_query_data_models` verifying insertion and retrieval of data model records.

### Phase 2: Data Model
- [ ] Task 2.1: Define `DataModel` struct in `src/index/data_models.rs` (or `src/impact/packet.rs`) with fields: `model_name`, `model_file_id` (i64), `language`, `model_kind`, `confidence` (f64), `evidence` (Option<String>). Derive `Serialize`, `Deserialize`, `Clone`, `Debug`.
- [ ] Task 2.2: Define `ModelKind` enum with variants: `STRUCT`, `INTERFACE`, `CLASS`, `SCHEMA`, `GENERATED`. Implement `Display` and `FromStr` for the enum.
- [ ] Task 2.3: Add `pub data_models: Vec<DataModel>` field to `ChangedFile` in `src/impact/packet.rs` with `#[serde(default)]` for backward compatibility.
- [ ] Task 2.4: Write unit tests verifying `DataModel` serialization/deserialization and `ChangedFile` backward compatibility (old JSON without `data_models` still parses).

### Phase 3: Data Model Detection - Rust
- [ ] Task 3.1: Add data model detection to `src/index/languages/rust.rs`:
  - Detect structs with `#[derive(Serialize, Deserialize)]` or `#[derive(serde::Serialize, serde::Deserialize)]`.
  - Detect structs with `#[derive(sqlx::FromRow)]` or `#[derive(diesel::Queryable)]`.
  - Detect structs with `#[serde(rename_all = "...")]` attribute.
  - Detect structs in `models/`, `entities/`, `schema/`, `domain/` directories (naming convention).
- [ ] Task 3.2: Implement detection confidence levels: derives (high confidence), directory naming (medium confidence).
- [ ] Task 3.3: Write unit tests for Rust data model detection: Serde struct, SQLx struct, directory-convention struct, plain struct (not a model).

### Phase 4: Data Model Detection - TypeScript
- [ ] Task 4.1: Add data model detection to `src/index/languages/typescript.rs`:
  - Detect interfaces/types in `models/`, `types/`, `schemas/`, `interfaces/` directories.
  - Detect classes extending `Model` (Sequelize, Objection.js).
  - Detect classes/interfaces with `@Entity` decorator (TypeORM).
- [ ] Task 4.2: Write unit tests for TypeScript data model detection: interface in models directory, class extending Model, interface in src directory (not a model).

### Phase 5: Data Model Detection - Python
- [ ] Task 5.1: Add data model detection to `src/index/languages/python.rs`:
  - Detect classes inheriting from `BaseModel` (Pydantic).
  - Detect classes inheriting from `Base` (SQLAlchemy declarative_base).
  - Detect classes inheriting from `db.Model` (Flask-SQLAlchemy).
  - Detect classes inheriting from `Model` (Django).
  - Detect classes decorated with `@dataclass` in `models/` directories.
  - Detect classes in `models.py` files or `models/` packages.
- [ ] Task 5.2: Write unit tests for Python data model detection: Pydantic `BaseModel` subclass, SQLAlchemy `Base` subclass, `@dataclass` in `models/`, plain class (not a model).

### Phase 6: Data Model Extraction Module
- [ ] Task 6.1: Create `src/index/data_models.rs` with `DataModelExtractor` struct that queries `project_symbols`, dispatches to language-specific detectors, and streams results to SQLite.
- [ ] Task 6.2: Implement detection orchestration: for each `project_symbols` entry of kind `struct_item`, `class_declaration`, or `interface_declaration`, apply the language-specific heuristics.
- [ ] Task 6.3: Implement generated-model detection: check `project_topology` (from E1-3) for `Generated` directory role. Mark models in generated directories for reduced risk weight.
- [ ] Task 6.4: Implement graceful skip: if `project_symbols` is empty, log info and return.
- [ ] Task 6.5: Write integration tests for `DataModelExtractor`: Rust project with Serde models, TypeScript project with interfaces, Python project with Pydantic models, empty project.

### Phase 7: Index Command Integration
- [ ] Task 7.1: Add data model extraction step to `changeguard index` after `project_symbols` is populated. Call `DataModelExtractor::extract()` with the database connection.
- [ ] Task 7.2: Add `--skip-data-models` flag to `changeguard index` for users who want indexing without data model detection.
- [ ] Task 7.3: Verify incremental indexing clears and rebuilds `data_models` only for re-indexed files.

### Phase 8: Impact Integration
- [ ] Task 8.1: In `src/impact/analysis.rs`, add a `data_model_risk` function that queries `data_models` (joining on `model_file_id`) for models whose `model_name` matches symbols in the changed files. Changed data models contribute up to 35 points within the Data Contract category (max 35 points). Generated models contribute up to 20 points.
- [ ] Task 8.2: Add risk reason: `"Data model: {model_name} ({model_kind})"`.
- [ ] Task 8.3: Integrate `data_model_risk` into the `analyze_risk` pipeline. If `data_models` table is empty, skip the query.
- [ ] Task 8.4: Populate the `data_models` field on `ChangedFile` during `impact` by querying data models for each changed file.
- [ ] Task 8.5: Write integration tests: changing a data model struct produces `"Data model: UserModel (STRUCT)"` risk reason and up to 35 points within the Data Contract category (max 35 points). Changing a generated model produces up to 20 points within the Data Contract category.

### Phase 9: End-to-End Testing
- [ ] Task 9.1: Create fixture Rust project with `#[derive(Serialize, Deserialize)] struct UserModel`. Run `changeguard index`, verify `data_models` contains the model.
- [ ] Task 9.2: Create fixture Python project with `class UserModel(BaseModel):`. Run `changeguard index`, verify `data_models` contains the model.
- [ ] Task 9.3: Create fixture TypeScript project with `interface User` in `models/user.ts`. Run `changeguard index`, verify `data_models` contains the model.
- [ ] Task 9.4: Run `changeguard impact` on a data model change. Verify JSON report includes `data_models` field and risk reason includes `"Data model"`.
- [ ] Task 9.5: Run `changeguard impact` on a repo without data models. Verify no data-model-related risk reasons and no regressions.
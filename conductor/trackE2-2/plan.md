## Plan: Track E2-2 - Framework-Aware Route Mapping

### Phase 1: Database Schema
- [ ] Task 1.1: Add `api_routes` table creation to Migration M16 in `src/state/migrations.rs` (shared with E2-1). Columns: `id`, `method`, `path_pattern`, `handler_symbol_id` (INTEGER REFERENCES project_symbols(id)), `handler_symbol_name` (TEXT, fallback), `handler_file_id` (INTEGER NOT NULL REFERENCES project_files(id)), `framework`, `route_source` (TEXT NOT NULL DEFAULT 'DECORATOR'), `mount_prefix` (TEXT, nullable), `is_dynamic` (INTEGER DEFAULT 0), `route_confidence` (REAL NOT NULL DEFAULT 1.0), `evidence` (TEXT, nullable), `last_indexed_at`. Include indices on `handler_symbol_id+handler_file_id` and `path_pattern`.
- [ ] Task 1.2: Add `api_routes` to the `test_all_tables_exist` test in `src/state/migrations.rs`.
- [ ] Task 1.3: Write a new test `test_insert_and_query_api_routes` verifying insertion and retrieval of route records.

### Phase 2: Data Model
- [ ] Task 2.1: Define `ApiRoute` struct in `src/impact/packet.rs` (or a new `src/index/routes.rs` module) with fields: `method`, `path_pattern`, `handler_symbol_id` (Option<i64>), `handler_symbol_name` (Option<String>), `handler_file_id` (i64), `framework`, `route_source`, `mount_prefix` (Option<String>), `is_dynamic` (bool), `route_confidence` (f64), `evidence` (Option<String>). Derive `Serialize`, `Deserialize`, `Clone`, `Debug`.
- [ ] Task 2.2: Add `pub api_routes: Vec<ApiRoute>` field to `ChangedFile` in `src/impact/packet.rs` with `#[serde(default)]` for backward compatibility.
- [ ] Task 2.3: Write unit tests verifying `ApiRoute` serialization/deserialization and `ChangedFile` backward compatibility (old JSON without `api_routes` still parses).

### Phase 3: Route Extraction - Rust (Actix, Axum, Rocket)
- [ ] Task 3.1: Add route extraction functions to `src/index/languages/rust.rs`:
  - `extract_actix_routes`: detect `#[get("/path")]`, `#[post("/path")]`, etc. attribute macros on handler functions.
  - `extract_axum_routes`: detect `.route("/path", get(handler))` patterns in route-setup functions.
  - `extract_rocket_routes`: detect `#[rocket::get("/path")]`, `#[rocket::post("/path")]` attribute macros.
- [ ] Task 3.2: Implement handler resolution: match handler function name against `project_symbols`.
- [ ] Task 3.3: Implement route-group prefix concatenation for Actix `web::scope` and Axum `Router::nest`.
- [ ] Task 3.4: Write unit tests for each Rust framework: Actix attribute routes, Axum method-chain routes, Rocket attribute routes, scope/prefix concatenation.

### Phase 4: Route Extraction - TypeScript (Express, Fastify)
- [ ] Task 4.1: Add route extraction functions to `src/index/languages/typescript.rs`:
  - `extract_express_routes`: detect `app.get("/path", handler)`, `app.post("/path", handler)`, `router.get("/path", handler)`.
  - `extract_fastify_routes`: detect `fastify.get("/path", handler)`, `fastify.post("/path", handler)`.
- [ ] Task 4.2: Implement handler resolution for TypeScript: match handler identifiers against `project_symbols`.
- [ ] Task 4.3: Implement Express Router prefix concatenation: `router.use("/prefix", subRouter)` patterns.
- [ ] Task 4.4: Write unit tests for Express route extraction: `app.get`, `app.post`, `router.get`, prefix concatenation.

### Phase 5: Route Extraction - Python (FastAPI, Flask)
- [ ] Task 5.1: Add route extraction functions to `src/index/languages/python.rs`:
  - `extract_fastapi_routes`: detect `@app.get("/path")`, `@app.post("/path")`, `@router.get("/path")` decorators.
  - `extract_flask_routes`: detect `@app.route("/path")`, `@app.route("/path", methods=["GET"])`, `@blueprint.route("/path")` decorators.
- [ ] Task 5.2: Implement handler resolution for Python: match decorated function name against `project_symbols`.
- [ ] Task 5.3: Implement FastAPI `APIRouter` prefix concatenation: `router = APIRouter(prefix="/api")` + `@router.get("/users")` = `/api/users`.
- [ ] Task 5.4: Implement Flask `Blueprint` prefix concatenation.
- [ ] Task 5.5: Write unit tests for FastAPI and Flask route extraction: decorator-based routes, prefix concatenation, parameterized routes.

### Phase 6: Route Extraction Module
- [ ] Task 6.1: Create `src/index/routes.rs` with `RouteExtractor` struct that dispatches to language-specific extractors and streams results to SQLite.
- [ ] Task 6.2: Implement framework detection: scan for framework imports/dependencies to determine which extractors to invoke (avoid running Flask extraction on a Rust project).
- [ ] Task 6.3: Implement dynamic route detection: routes with variable interpolation or runtime construction are labeled with `path_pattern = "DYNAMIC"`, `is_dynamic = 1`, and `route_confidence = 0.5`.
- [ ] Task 6.4: Implement graceful skip: if no framework patterns are detected, return empty results without error.
- [ ] Task 6.5: Write integration tests for `RouteExtractor`: multi-framework repo, dynamic routes, missing framework, empty project.

### Phase 7: Index Command Integration
- [ ] Task 7.1: Add route extraction step to `changeguard index` after `project_symbols` is populated. Call `RouteExtractor::extract()` with the database connection.
- [ ] Task 7.2: Add `--skip-routes` flag to `changeguard index` for users who want indexing without route mapping.
- [ ] Task 7.3: Verify incremental indexing clears and rebuilds `api_routes` only for re-indexed files.

### Phase 8: Impact Integration
- [ ] Task 8.1: In `src/impact/analysis.rs`, add a `route_risk` function that queries `api_routes` for routes whose handler symbols appear in the changed files. Route handlers contribute up to 30 points within the API Surface category (max 35 points). Add risk reason: `"Public API route: {METHOD} {path_pattern}"`.
- [ ] Task 8.2: Integrate `route_risk` into the `analyze_risk` pipeline. If `api_routes` table is empty, skip the query.
- [ ] Task 8.3: Populate the `api_routes` field on `ChangedFile` during `impact` by querying routes for each changed file's handler symbols.
- [ ] Task 8.4: Write integration tests: changing a route handler produces `"Public API route: GET /users"` risk reason and up to 30 points within the API Surface category (max 35 points).

### Phase 9: End-to-End Testing
- [ ] Task 9.1: Create fixture Rust/Actix project with `#[get("/users")]` handler. Run `changeguard index`, verify `api_routes` contains the route.
- [ ] Task 9.2: Create fixture Express project with `app.get("/api/users", getUsers)`. Run `changeguard index`, verify `api_routes` contains the route.
- [ ] Task 9.3: Create fixture FastAPI project with `@router.get("/items/{item_id}")`. Run `changeguard index`, verify `api_routes` contains the parameterized route.
- [ ] Task 9.4: Run `changeguard impact` on a route handler change. Verify JSON report includes `api_routes` and risk reason includes `"Public API route"`.
- [ ] Task 9.5: Run `changeguard impact` on a repo without web frameworks. Verify no route-related risk reasons and no regressions.
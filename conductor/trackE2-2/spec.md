# Specification: Track E2-2 - Framework-Aware Route Mapping

## 1. Objective
Detect web framework routing patterns (Actix, Axum, Rocket, Express, FastAPI, Flask) in source code and map URL patterns to handler functions. Store results in a new `api_routes` table (Migration M16) and integrate with risk scoring (+30 for changed route handlers) and impact reports. This track depends on E1-1 (project_symbols) for handler resolution and runs in parallel with E2-3 after E2-1 lands.

## 2. Deliverables

### 2.1 Route Extraction via Tree-Sitter Queries and Pattern Matching
- **Target files**: `src/index/languages/rust.rs`, `src/index/languages/typescript.rs`, `src/index/languages/python.rs`
- **Details**: Add route extraction to each language module. Detect framework-specific routing patterns:
  - **Rust (Actix Web)**: `#[actix_web::get("/path")]`, `#[actix_web::post("/path")]`, `#[actix_web::put("/path")]`, `#[actix_web::delete("/path")]`, `#[get("/path")]`, `#[post("/path")]` attribute macros on handler functions. Also detect `web::resource("/path").route(web::get().to(handler))` patterns.
  - **Rust (Axum)**: `Router::new().route("/path", get(handler))`, `Router::new().route("/path", post(handler))` method chains in route-setup functions. Detect `.route(` calls on `Router` values.
  - **Rust (Rocket)**: `#[rocket::get("/path")]`, `#[rocket::post("/path")]` attribute macros.
  - **TypeScript (Express/Koa)**: `app.get("/path", handler)`, `app.post("/path", handler)`, `router.get("/path", handler)`, `router.post("/path", handler)`. Detect method-chain patterns on `Router` and `Application` objects.
  - **TypeScript (Fastify)** *(enhancement beyond expansion plan)*: `fastify.get("/path", handler)`, `fastify.post("/path", handler)` patterns. Also detect `fastify.register(routes, { prefix: "/path" })` mounted routers. Fastify support is included as a high-value enhancement given its growing adoption and the structural similarity to Express extraction.
  - **Python (FastAPI)**: `@app.get("/path")`, `@app.post("/path")`, `@router.get("/path")`, `@router.post("/path")` decorators. Detect `APIRouter()` assignments.
  - **Python (Flask)**: `@app.route("/path")`, `@app.route("/path", methods=["GET"])`, `@blueprint.route("/path")` decorators.

### 2.2 API Routes Table (Migration M16)
- **Target file**: `src/state/migrations.rs`
- **Details**: Add to Migration M16 (shared with E2-1) creating the `api_routes` table:
  ```sql
  CREATE TABLE IF NOT EXISTS api_routes (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      method              TEXT NOT NULL,
      path_pattern        TEXT NOT NULL,
      handler_symbol_id   INTEGER REFERENCES project_symbols(id),
      handler_symbol_name TEXT,           -- fallback: raw handler name when not in project_symbols
      handler_file_id     INTEGER NOT NULL REFERENCES project_files(id),
      framework           TEXT NOT NULL,
      route_source        TEXT NOT NULL DEFAULT 'DECORATOR',  -- DECORATOR, ROUTER_CHAIN, APP_METHOD, MOUNTED_ROUTER
      mount_prefix        TEXT,           -- e.g., "/api/v1" when a router is mounted at a prefix
      is_dynamic          INTEGER DEFAULT 0,  -- 1 if path_pattern is "DYNAMIC"
      route_confidence    REAL NOT NULL DEFAULT 1.0,
      evidence            TEXT,
      last_indexed_at     TEXT NOT NULL,
      FOREIGN KEY (handler_symbol_id) REFERENCES project_symbols(id),
      FOREIGN KEY (handler_file_id) REFERENCES project_files(id)
  );
  CREATE INDEX IF NOT EXISTS idx_api_routes_handler
      ON api_routes(handler_symbol_id, handler_file_id);
  CREATE INDEX IF NOT EXISTS idx_api_routes_path
      ON api_routes(path_pattern);
  ```
- The `method` field stores HTTP methods: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`, `ALL`.
- The `framework` field stores: `actix`, `axum`, `rocket`, `express`, `fastify`, `flask`, `fastapi`.
- The `route_source` field indicates how the route was detected: `DECORATOR` (attribute/decorator on handler), `ROUTER_CHAIN` (method chain on a Router object), `APP_METHOD` (direct app.get/post call), `MOUNTED_ROUTER` (router mounted at a prefix).
- The `handler_symbol_name` field stores the raw handler name as a fallback when `handler_symbol_id` is NULL (handler not found in `project_symbols`).
- The `route_confidence` field stores a value between 0.0 and 1.0. Decorator-detected routes default to 1.0. Dynamically-constructed routes default to 0.5.
- The `evidence` field stores an optional JSON string describing what was observed (e.g., `"#[get(\"/users\")] on get_users"`, `"app.get(\"/api/users\", getUsers)"`).

### 2.3 Route Extraction Module
- **Target file**: New `src/index/routes.rs`
- **Details**: Implement `RouteExtractor` that:
  1. Iterates over source files, dispatching to language-specific route extractors.
  2. For each detected route, resolves the handler function name against `project_symbols`. Resolved handlers produce `handler_symbol_id`; unresolved handlers store the raw name in `handler_symbol_name` with `handler_symbol_id = NULL`.
  3. Handles route group/prefix concatenation: if a Router/Blueprint has a prefix, prepend it to all child route patterns. Store the prefix in `mount_prefix`.
  4. Marks dynamically-constructed routes as `path_pattern = "DYNAMIC"` with `is_dynamic = 1`, `route_confidence = 0.5`, and `route_source` indicating the detection method.
  5. Populates `route_source` based on detection method: `DECORATOR` for attribute/decorator routes, `ROUTER_CHAIN` for method chains, `APP_METHOD` for direct app calls, `MOUNTED_ROUTER` for mounted sub-routers.
  6. Populates `evidence` with a brief description of what was observed (e.g., `"#[get(\"/users\")] on get_users"`, `"app.get(\"/api/users\", getUsers)"`).
  7. Streams extracted routes to SQLite.

### 2.4 Impact Integration
- **Target file**: `src/impact/analysis.rs`
- **Details**:
  - When a changed file contains route handler symbols, query `api_routes` for routes pointing to those handlers (matching on `handler_symbol_id` or `handler_symbol_name`).
  - Route handlers contribute up to 30 points within the API Surface category (max 35 points).
  - Add risk reason: `"Public API route: {METHOD} {path_pattern}"`.
  - Extend `ChangedFile` in `src/impact/packet.rs` with an `api_routes: Vec<ApiRoute>` field (with `#[serde(default)]` for backward compatibility).

### 2.5 Index Command Integration
- **Target file**: Command handler for `changeguard index`
- **Details**: Route extraction runs after `project_symbols` is populated (handler resolution depends on symbol data). Invoked as part of the `changeguard index` pipeline.

### 2.6 LSP Integration (Future)
- The expansion plan specifies route annotations in CodeLens as a future deliverable. This track stores the data but does not implement the LSP integration (that is a separate cross-cutting concern).

## 3. Constraints & Guidelines
- **Deterministic over speculative**: Routes detected with low confidence (e.g., dynamically constructed paths) must be labeled with `path_pattern = "DYNAMIC"`. Never present a guessed URL as a confirmed route.
- **Graceful degradation**: If no framework patterns are detected, the `api_routes` table remains empty and `impact` proceeds normally.
- **No performance regression**: Route extraction must add less than 2 seconds to `changeguard index` for a 2000-file repo.
- **Multiple frameworks**: A repo may use multiple frameworks (e.g., an Express API server alongside a Python FastAPI worker). Detect and tag each route with its framework.
- **Backward-compatible schema**: The `api_routes` table is additive. No existing table is modified.
- **Route group/prefix handling**: When a framework supports route groups (Actix `web::scope`, Express `Router()`, FastAPI `APIRouter`, Flask `Blueprint`), concatenate the group prefix with the route pattern to produce the full path.
- **Config-defined routes**: Routes defined in YAML/TOML config files (e.g., OpenAPI specs, serverless.yml) are explicitly out of scope for this phase. Document as a known gap.

## 4. Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Dynamic route construction (e.g., `app.use(dynamicPrefix, router)`) | Store with `path_pattern = "DYNAMIC"`, `is_dynamic = 1`, `route_confidence = 0.5`. Log as info. |
| Routes defined in configuration files (YAML, TOML) | Skip. Document as a known gap for future phases. |
| Multiple frameworks in one repo | Detect all. Tag each route with its `framework`. |
| Route groups/prefixes | Concatenate group prefix with route pattern. Store prefix in `mount_prefix`. E.g., `APIRouter(prefix="/api")` + `@router.get("/users")` = `/api/users`. |
| Parameterized routes (e.g., `/users/:id`) | Store the pattern as-is with the parameter syntax (`:id`, `{id}`, `<id>`). Do not attempt to enumerate concrete paths. |
| Middleware-only handlers (no route) | Skip. Only extract handlers that are bound to an HTTP method + path. |
| Handler function not in `project_symbols` | Set `handler_symbol_id = NULL`. Store the raw handler name in `handler_symbol_name`. The route is still recorded. |
| Same handler bound to multiple routes | Store each route-handler pair as a separate row. |
| Routes in generated code (e.g., protobuf services) | Skip files in `Generated` directories (from E1-3 topology classification). |
| No web framework in project | `api_routes` table remains empty. `impact` proceeds without route-based risk scoring. |
| Fastify registered routes with prefix | Detect `fastify.register(routes, { prefix: "/path" })`. Set `route_source = 'MOUNTED_ROUTER'` and `mount_prefix = "/path"`. |

## 5. Acceptance Criteria

1. `changeguard index` populates `api_routes` for Actix, Axum, Rocket, Express, Fastify, FastAPI, and Flask projects.
2. `changeguard impact` shows API routes in the JSON report when a changed file contains route handlers.
3. Changed route handlers receive risk weight up to 30 points within the API Surface category (max 35 points).
4. Risk reason for a changed route handler includes the HTTP method and path pattern: `"Public API route: GET /users"`.
5. `ChangedFile.api_routes` field appears in serialized `ImpactPacket` JSON output.
6. Routes with dynamically constructed paths are labeled `"DYNAMIC"`, not as confirmed paths.
7. Repos without web frameworks produce empty `api_routes` and no warnings or errors.

## 6. Verification Gate

- **Fixture test (Rust/Actix)**: A Rust project with `#[get("/users")]` on `get_users` handler extracts route `GET /users` mapped to `get_users`.
- **Fixture test (Rust/Axum)**: A Rust project with `.route("/api/users", get(list_users))` extracts route `GET /api/users` mapped to `list_users`.
- **Fixture test (TypeScript/Express)**: An Express project with `app.get("/api/users", getUsers)` extracts route `GET /api/users` mapped to `getUsers`.
- **Fixture test (Python/FastAPI)**: A FastAPI project with `@router.get("/items/{item_id}")` extracts route `GET /items/{item_id}` mapped to the handler.
- **Impact test**: Changing a route handler function produces a risk reason `"Public API route: GET /users"` and up to 30 points within the API Surface category (max 35 points).
- **JSON report test**: The `api_routes` field appears in the serialized `ImpactPacket` for changed files containing route handlers.
- **Empty-table test**: With no `api_routes` data, `impact` produces output identical to the baseline.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M16 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E2 tables for fixture repos
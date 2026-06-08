# Implementation Plan: API Endpoint Ownership & Consumer Graph (Track W2)

### Phase 1: Core Graph and Data Model Upgrades
- [ ] Task 1.1: Update `src/state/graph_kinds.rs` to include `EdgeKind::Consumes`. Ensure `NodeKind::Endpoint` and `EdgeKind::Authenticates`, `Authorizes`, `Owns`, `Handles` are fully documented and supported in migrations.
- [ ] Task 1.2: Add SQLite migration (e.g., `m34_api_route_enrichment.rs`) to alter `api_routes` table, adding `auth_requirements`, `schema_refs`, `owning_service`, and `consumers` columns as TEXT (JSON).
- [ ] Task 1.3: Update `ExtractedRoute` and `RouteRow` structs in `src/index/routes.rs` to parse and hold the new metadata.

### Phase 2: AST Extraction Enhancements (Rust)
- [ ] Task 2.1: Enhance `collect_rust_routes` in `src/index/languages/rust/routes.rs` to extract authentication middleware from Axum (`.layer`, `.route_layer`) and Actix (`.wrap`).
- [ ] Task 2.2: Add Rocket request guard parsing to identify auth-related function parameters in handlers.
- [ ] Task 2.3: Extract input schema types (e.g., `Json<T>`, `Form<T>`) from handler signatures and map them to `schema_refs`.
- [ ] Task 2.4: Distinguish `unknown` vs `public` vs `secured` auth states correctly during AST analysis.

### Phase 3: Consumer Detection & Graph Joining
- [ ] Task 3.1: Implement consumer detection in `src/index/references.rs` or a new worker, searching for `reqwest`/`hyper` calls that match indexed endpoint paths.
- [ ] Task 3.2: Update `src/index/graph_worker.rs` to sink `api_routes` into CozoDB, constructing `NodeKind::Endpoint` and all related edges (`OWNS`, `HANDLES`, `AUTHENTICATES`, `CONSUMES`).

### Phase 4: Impact Analysis & Risk Scoring
- [ ] Task 4.1: Create/Update `src/impact/enrichment/api.rs` to detect removed endpoints or changed auth states between snapshots.
- [ ] Task 4.2: Add impact traversal logic that flags consumer symbols/services when an endpoint contract is modified or deleted.

### Phase 5: CLI & User Interfaces
- [ ] Task 5.1: Add `Endpoints` subcommand to `src/cli.rs` and `src/commands/mod.rs`.
- [ ] Task 5.2: Implement `src/commands/endpoints.rs` to query CozoDB for endpoints, format results into a table (Method, Path, Service, Auth, Consumers) or JSON output.
- [ ] Task 5.3: Verify documentation and help strings for the new command.
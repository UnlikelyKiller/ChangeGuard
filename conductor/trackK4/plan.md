# Track K4: Service Boundary & Communication Plan

## Phase 1: Boundary Extraction
- [x] Add `BoundaryMarker` enum: `CargoWorkspace`, `NpmPackage`, `GoModule`, `Dockerfile`.
- [x] Implement `BoundaryDetector` in `src/coverage/services.rs`:
    - [x] Recursively walk tree for markers.
    - [x] Map directories to logical service names.
- [x] Store boundaries in CozoDB relation `service_roots`.

## Phase 2: Communication Extraction
- [x] Extend tree-sitter extractors (Rust, TS, Python) to detect client patterns:
    - [x] Rust: `ureq::post`, `reqwest::get`.
    - [x] TS: `fetch`, `axios.get`.
    - [x] Python: `requests.get`, `httpx.post`.
- [x] Resolve client targets to known `api_routes` in other service roots.
- [x] Store edges in `service_dependencies` relation.

## Phase 3: Impact Integration
- [x] Implement `enrich_service_impact` in `ImpactOrchestrator`.
- [x] Flag risk if a public route in a service root is modified and has external consumers.
- [x] Update `viz` to support `--view services` (grouped nodes).

## Phase 4: Final Verification
- [x] Create monorepo test fixture with `OrderService` (consumer) and `InventoryService` (provider).
- [x] Verify `InventoryService` change flags `OrderService` impact.
- [x] CI Gate.

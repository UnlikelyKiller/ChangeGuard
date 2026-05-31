# Track K4: Service Boundary & Communication Plan

## Phase 1: Boundary Extraction
- [ ] Add `BoundaryMarker` enum: `CargoWorkspace`, `NpmPackage`, `GoModule`, `Dockerfile`.
- [ ] Implement `BoundaryDetector` in `src/coverage/services.rs`:
    - [ ] Recursively walk tree for markers.
    - [ ] Map directories to logical service names.
- [ ] Store boundaries in CozoDB relation `service_roots`.

## Phase 2: Communication Extraction
- [ ] Extend tree-sitter extractors (Rust, TS, Python) to detect client patterns:
    - [ ] Rust: `ureq::post`, `reqwest::get`.
    - [ ] TS: `fetch`, `axios.get`.
    - [ ] Python: `requests.get`, `httpx.post`.
- [ ] Resolve client targets to known `api_routes` in other service roots.
- [ ] Store edges in `service_dependencies` relation.

## Phase 3: Impact Integration
- [ ] Implement `enrich_service_impact` in `ImpactOrchestrator`.
- [ ] Flag risk if a public route in a service root is modified and has external consumers.
- [ ] Update `viz` to support `--view services` (grouped nodes).

## Phase 4: Final Verification
- [ ] Create monorepo test fixture with `OrderService` (consumer) and `InventoryService` (provider).
- [ ] Verify `InventoryService` change flags `OrderService` impact.
- [ ] CI Gate.

## Plan: Track M7-2 — Service-Map Derivation

### Phase 1: Service Inference
- [ ] Task 1.1: Implement `infer_services()` in `src/coverage/services.rs`.
- [ ] Task 1.2: Implement multi-strategy naming: directory → package name → fallback.
- [ ] Task 1.3: Implement monorepo depth cap at 2 levels.
- [ ] Task 1.4: Write test: routes in `src/api/users/` → service "users".
- [ ] Task 1.5: Write test: flat repo `src/handler.rs` → single service "src".
- [ ] Task 1.6: Write test: package-name fallback from `package.json` parent.
- [ ] Task 1.7: Write test: unnamed-service fallback when no directory or package.
- [ ] Task 1.8: Write test: empty routes → returns empty vec.

### Phase 2: Cross-Service Edges
- [ ] Task 2.1: Implement `compute_cross_service_edges()`.
- [ ] Task 2.2: Implement edge deduplication with counts.
- [ ] Task 2.3: Write test: cross-service edge computed from call graph.
- [ ] Task 2.4: Write test: 5 A→B edges collapsed to one with count=5.
- [ ] Task 2.5: Write test: same-service edges filtered out.

### Phase 3: ServiceMapDelta Type
- [ ] Task 3.1: Define `Service` and `ServiceMapDelta` types.
- [ ] Task 3.2: Add `service_map_delta: Option<ServiceMapDelta>` to `ImpactPacket`.
- [ ] Task 3.3: Write test: `Service` serialization roundtrip.
- [ ] Task 3.4: Write test: `ServiceMapDelta` with `None` → field absent in JSON.

### Phase 4: Index Integration
- [ ] Task 4.1: Wire service inference into `changeguard index` pipeline.
- [ ] Task 4.2: Store service assignments in a queryable structure (in-memory or derived).
- [ ] Task 4.3: Write test: index run produces service assignments for test repo.

### Phase 5: Risk Enrichment
- [ ] Task 5.1: Compute `ServiceMapDelta` during `impact` from pre-indexed data.
- [ ] Task 5.2: Implement risk escalation: 2 svcs=Low, 3-4=Medium, 5+=High.
- [ ] Task 5.3: Write test: 2-service change → Low elevation.
- [ ] Task 5.4: Write test: 4-service change → Medium elevation.
- [ ] Task 5.5: Write test: 5-service change → High elevation.
- [ ] Task 5.6: Write test: `[coverage.services].enabled = false` → no enrichment.

### Phase 6: Final Validation
- [ ] Task 6.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 6.2: Run `cargo test coverage::services` — all tests pass.
- [ ] Task 6.3: Run full `cargo test` — no regressions.

## Plan: Track M7-2 — Service-Map Derivation

### Phase 1: Service Inference
- [x] Task 1.1: Implement `infer_services()` in `src/coverage/services.rs`.
- [x] Task 1.2: Implement multi-strategy naming: directory → package name → fallback.
- [x] Task 1.3: Implement monorepo depth cap (default 2).
- [x] Task 1.4: Write test: `src/api/users/mod.rs` → service `users`.
- [x] Task 1.5: Write test: depth 3 `src/api/users/auth/mod.rs` → capped to `users`.
- [x] Task 1.6: Write test: package name fallback (find `Cargo.toml` in parent).

### Phase 2: Cross-Service Edges
- [x] Task 2.1: Implement `compute_cross_service_edges()` in `src/coverage/services.rs`.
- [x] Task 2.2: Implement edge deduplication and counting.
- [x] Task 2.3: Filter out same-service edges.
- [x] Task 2.4: Write test: multi-edge collapse.
- [x] Task 2.5: Write test: service mapping from call graph.

### Phase 3: Types and ImpactPacket Update
- [x] Task 3.1: Define `ServiceMapDelta` and `Service` in `src/impact/packet.rs`.
- [x] Task 3.2: Update `ImpactPacket` to include `service_map_delta: Option<ServiceMapDelta>`.
- [x] Task 3.3: Update all `ImpactPacket` initializations.

### Phase 4: Index Integration
- [x] Task 4.1: Add migration M15 to add `service_name` to `project_files`.
- [x] Task 4.2: Add `infer_services` method to `ProjectIndexer` in `src/index/project_index.rs`.
- [x] Task 4.3: Update `src/commands/index.rs` to call `indexer.infer_services()`.
- [x] Task 4.4: Add service stats to index output.

### Phase 5: Risk Enrichment
- [x] Task 5.1: Implement `populate_service_map()` in `src/commands/impact.rs`.
- [x] Task 5.2: Detect affected services from changed files.
- [x] Task 5.3: Compute cross-service edges for current changes.
- [x] Task 5.4: Elevate risk: 2 services → Elevated, 3+ services → High.
- [x] Task 5.5: Add risk reasons with service names.

### Phase 6: Final Validation
- [x] Task 6.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Task 6.2: Run `cargo test coverage::services` — all tests pass.
- [x] Task 6.3: Run full `cargo test` — no regressions.

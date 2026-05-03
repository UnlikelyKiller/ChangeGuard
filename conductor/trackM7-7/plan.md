## Plan: Track M7-7 — Impact Packet Extension & Enrichment Integration

### Phase 1: CoverageConfig
- [ ] Task 1.1: Add `CoverageConfig` and all sub-config types to `src/config/model.rs`.
- [ ] Task 1.2: All fields default to `enabled = false` via `#[serde(default)]`.
- [ ] Task 1.3: Wire `CoverageConfig` into main `Config` struct.
- [ ] Task 1.4: Write test: `CoverageConfig::default()` has all dimensions disabled.
- [ ] Task 1.5: Write test: M6-era config (no `[coverage]` section) deserializes with all disabled.

### Phase 2: ImpactPacket Fields
- [ ] Task 2.1: Add `trace_config_drift: Vec<TraceConfigChange>` with serde defaults.
- [ ] Task 2.2: Add `sdk_dependencies_delta: SdkDependencyDelta` with serde defaults.
- [ ] Task 2.3: Add `service_map_delta: Option<ServiceMapDelta>` with serde defaults.
- [ ] Task 2.4: Add `data_flow_matches: Vec<DataFlowMatch>` with serde defaults.
- [ ] Task 2.5: Add `deploy_manifest_changes: Vec<DeployManifestChange>` with serde defaults.
- [ ] Task 2.6: Add `staleness_days: Option<u32>` + `staleness_tier: Option<StalenessTier>` to `RelevantDecision`.
- [ ] Task 2.7: Write test: serialization roundtrip all fields populated.
- [ ] Task 2.8: Write test: serialization roundtrip all fields empty.
- [ ] Task 2.9: Write test: empty Vec fields absent from JSON output.

### Phase 3: finalize() & truncate_for_context()
- [ ] Task 3.1: Add sort calls in `finalize()` for all new Vec fields.
- [ ] Task 3.2: Add clear calls in `truncate_for_context()` Phase 3 for all new Vec fields.
- [ ] Task 3.3: Write test: `trace_config_drift` sorted by config_type.
- [ ] Task 3.4: Write test: `data_flow_matches` sorted by change_pct descending.
- [ ] Task 3.5: Write test: `deploy_manifest_changes` sorted by risk_tier descending.
- [ ] Task 3.6: Write test: all M7 fields cleared after `truncate_for_context()`.
- [ ] Task 3.7: Write test: `SdkDependencyDelta` inner Vecs sorted.

### Phase 4: Enrichment Hooks
- [ ] Task 4.1: Implement `enrich_trace_configs()` — M7-1 integration.
- [ ] Task 4.2: Implement `enrich_sdk_dependencies()` — M7-1 integration.
- [ ] Task 4.3: Implement `enrich_service_map()` — M7-2 integration.
- [ ] Task 4.4: Implement `enrich_data_flow()` — M7-3 integration.
- [ ] Task 4.5: Implement `enrich_deploy_manifests()` — M7-4 integration.
- [ ] Task 4.6: Implement `enrich_ci_self_awareness()` — M7-5 integration.
- [ ] Task 4.7: Implement `enrich_adr_staleness()` — M7-6 integration.
- [ ] Task 4.8: Wire all hooks into `execute_impact()` with per-dimension enabled checks.
- [ ] Task 4.9: Wire master kill switch `coverage.enabled`.
- [ ] Task 4.10: Write test: master kill switch → zero M7 enrichment.
- [ ] Task 4.11: Write test: per-dimension toggle → only that dimension populates.
- [ ] Task 4.12: Write test: enrichment runs after analyze_risk (risk reasons preserved).

### Phase 5: Risk Weighting
- [ ] Task 5.1: Wire risk weights per signal type into `analyze_risk()` scoring.
- [ ] Task 5.2: Implement weight caps for each signal type.
- [ ] Task 5.3: Write test: trace config weight capped at 9 (10 files changed).
- [ ] Task 5.4: Write test: data_flow weight capped at 20 (6 matches).
- [ ] Task 5.5: Write test: deploy manifest weight capped at 15 (6 manifests).

### Phase 6: Human Output
- [ ] Task 6.1: Add "Observability Config Drift" section to `print_impact_summary()`.
- [ ] Task 6.2: Add "SDK Dependencies" section.
- [ ] Task 6.3: Add "Service Map" section.
- [ ] Task 6.4: Add "Data-Flow Coupling" section.
- [ ] Task 6.5: Add "Deployment Manifests" section.
- [ ] Task 6.6: Add "CI Pipeline Impact" section.
- [ ] Task 6.7: Add "ADR Staleness" section (inline with existing "Relevant Documentation").
- [ ] Task 6.8: Write test: each section renders when field non-empty.
- [ ] Task 6.9: Write test: each section absent when field empty.

### Phase 7: Ask Context
- [ ] Task 7.1: Add M7 enrichment blocks to ask context assembly.
- [ ] Task 7.2: Implement budget enforcement: drop M7 sections as lowest priority.
- [ ] Task 7.3: Write test: M7 sections present in context when populated.
- [ ] Task 7.4: Write test: M7 sections dropped when budget exhausted.

### Phase 8: Final Validation
- [ ] Task 8.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 8.2: Run full `cargo test` — no regressions, all M7 tests pass.
- [ ] Task 8.3: Verify config backward compat: load M6 config with M7 binary, behavior identical.
- [ ] Task 8.4: Verify no hot-path embedding: grep enrichment hooks for `embed_long_text`/`embed_batch`.

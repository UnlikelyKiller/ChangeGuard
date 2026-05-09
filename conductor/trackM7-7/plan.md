## Plan: Track M7-7 — Impact Packet Extension & Enrichment Integration

### Phase 1: CoverageConfig
- [x] Task 1.1: Add `CoverageConfig` and all sub-config types to `src/config/model.rs`.
- [x] Task 1.2: All fields default to `enabled = false` via `#[serde(default)]`.
- [x] Task 1.3: Wire `CoverageConfig` into main `Config` struct.
- [x] Task 1.4: Write test: `CoverageConfig::default()` has all dimensions disabled.
- [x] Task 1.5: Write test: M6-era config (no `[coverage]` section) deserializes with all disabled.

### Phase 2: ImpactPacket Fields
- [x] Task 2.1: Add `trace_config_drift: Vec<TraceConfigChange>` with serde defaults.
- [x] Task 2.2: Add `sdk_dependencies_delta: SdkDependencyDelta` with serde defaults.
- [x] Task 2.3: Add `service_map_delta: Option<ServiceMapDelta>` with serde defaults.
- [x] Task 2.4: Add `data_flow_matches: Vec<DataFlowMatch>` with serde defaults.
- [x] Task 2.5: Add `deploy_manifest_changes: Vec<DeployManifestChange>` with serde defaults.
- [x] Task 2.6: Add `staleness_days: Option<u32>` + `staleness_tier: Option<StalenessTier>` to `RelevantDecision`.
- [x] Task 2.7: Write test: serialization roundtrip all fields populated.
- [x] Task 2.8: Write test: serialization roundtrip all fields empty.
- [x] Task 2.9: Write test: empty Vec fields absent from JSON output.
- [x] Task 2.10: Add `ci_config_change: Option<CiConfigChange>` to `ImpactPacket`.
- [x] Task 2.11: Move `CiConfigChange` struct to `packet.rs` with serde support.
- [x] Task 2.12: Write test: `ci_config_change` serialization roundtrip.
- [x] Task 2.13: Write test: `ci_config_change` absent from JSON when None.

### Phase 3: finalize() & truncate_for_context()
- [x] Task 3.1: Add sort calls in `finalize()` for all new Vec fields.
- [x] Task 3.2: Add clear calls in `truncate_for_context()` Phase 3 for all new Vec fields.
- [x] Task 3.3: Write test: `trace_config_drift` sorted by config_type.
- [x] Task 3.4: Write test: `data_flow_matches` sorted by change_pct descending.
- [x] Task 3.5: Write test: `deploy_manifest_changes` sorted by risk_tier descending.
- [x] Task 3.6: Write test: all M7 fields cleared after `truncate_for_context()`.
- [x] Task 3.7: Write test: `SdkDependencyDelta` inner Vecs sorted.
- [x] Task 3.8: Write test: `ci_config_change` cleared after `truncate_for_context()`.

### Phase 4: Enrichment Hooks
- [x] Task 4.1: Implement `enrich_trace_configs()` — M7-1 integration.
- [x] Task 4.2: Implement `enrich_sdk_dependencies()` — M7-1 integration.
- [x] Task 4.3: Implement `enrich_service_map()` — M7-2 integration.
- [x] Task 4.4: Implement `enrich_data_flow()` — M7-3 integration.
- [x] Task 4.5: Implement `enrich_deploy_manifests()` — M7-4 integration.
- [x] Task 4.6: Implement `enrich_ci_self_awareness()` — M7-5 integration.
- [x] Task 4.7: Implement `enrich_adr_staleness()` — M7-6 integration.
- [x] Task 4.8: Wire all hooks into `execute_impact()` with per-dimension enabled checks.
- [x] Task 4.9: Wire master kill switch `coverage.enabled`.
- [x] Task 4.10: Write test: master kill switch → zero M7 enrichment.
- [x] Task 4.11: Write test: per-dimension toggle → only that dimension populates.
- [x] Task 4.12: Write test: enrichment runs after analyze_risk (risk reasons preserved).
- [x] Task 4.13: Write test: `CISelfAwarenessProvider` populates `ci_config_change`.
- [x] Task 4.14: Write test: `CISelfAwarenessProvider` respects coverage master switch.
- [x] Task 4.15: Write test: `CISelfAwarenessProvider` respects per-dimension switch.

### Phase 5: Risk Weighting
- [x] Task 5.1: Wire risk weights per signal type into `analyze_risk()` scoring.
- [x] Task 5.2: Implement weight caps for each signal type.
- [x] Task 5.3: Write test: trace config weight capped at 9 (10 files changed).
- [x] Task 5.4: Write test: data_flow weight capped at 20 (6 matches).
- [x] Task 5.5: Write test: deploy manifest weight capped at 15 (6 manifests).
- [x] Task 5.6: Write test: CI self-awareness reads from `packet.ci_config_change` with fallback.

### Phase 6: Human Output
- [x] Task 6.1: Add "Observability Config Drift" section to `print_impact_summary()`.
- [x] Task 6.2: Add "SDK Dependencies" section.
- [x] Task 6.3: Add "Service Map" section.
- [x] Task 6.4: Add "Data-Flow Coupling" section.
- [x] Task 6.5: Add "Deployment Manifests" section.
- [x] Task 6.6: Add "CI Pipeline Impact" section.
- [x] Task 6.7: Add "ADR Staleness" section (inline with existing "Relevant Documentation").
- [x] Task 6.8: Write test: each section renders when field non-empty.
- [x] Task 6.9: Write test: each section absent when field empty.

### Phase 7: Ask Context
- [x] Task 7.1: Add M7 enrichment blocks to ask context assembly (implicit via ImpactPacket JSON).
- [x] Task 7.2: Implement budget enforcement: drop M7 sections as lowest priority (via `truncate_for_context()`).
- [x] Task 7.3: Write test: M7 sections present in context when populated.
- [x] Task 7.4: Write test: M7 sections dropped when budget exhausted.

### Phase 8: Final Validation
- [x] Task 8.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Task 8.2: Run full `cargo test` — no regressions, all M7 tests pass.
- [x] Task 8.3: Verify config backward compat: load M6 config with M7 binary, behavior identical.
- [x] Task 8.4: Verify no hot-path embedding: grep enrichment hooks for `embed_long_text`/`embed_batch`.

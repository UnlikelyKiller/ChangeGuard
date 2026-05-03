# Audit 7 — Milestone M7 Implementation Audit

**Date:** 2026-05-03  
**Branch:** track-M1-2  
**Auditor:** Claude (automated)  
**Reference:** `docs/observability-plan2.md`, `conductor/trackM7-{1..7}/`

---

## Executive Summary

All seven M7 tracks have core infrastructure in place: detection modules exist, config sections are wired, ImpactPacket carries all five new fields, all seven enrichment hooks are called from `execute_impact`, `finalize()` sorts the new fields, and `truncate_for_context()` clears them. The critical unresolved gap is **human-readable output** — `print_impact_summary()` has zero coverage of any M7 field. Additionally, `analyze_risk()` hardcodes several config-tunable risk weights, `enrich_ci_self_awareness` is a no-op stub, and test coverage for detection logic is sparse.

| Track | Core Detection | Config | Packet Fields | Hooks Wired | Risk Scoring | Human Output | Tests |
|-------|---------------|--------|---------------|-------------|--------------|--------------|-------|
| M7-1 Trace & SDK | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ |
| M7-2 Service Map | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ |
| M7-3 Data Flow | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ |
| M7-4 Deploy Manifests | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ |
| M7-5 CI Self-Awareness | ⚠️ stub | ✅ | n/a | ⚠️ stub | ✅ | ❌ | ⚠️ |
| M7-6 ADR Staleness | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| M7-7 Packet Extension | ✅ | n/a | ✅ | ✅ | ✅ | ❌ | — |

---

## Track-by-Track Findings

### M7-1: Trace Config & SDK Detection

**Status: SUBSTANTIALLY IMPLEMENTED — human output and M7-specific risk tests missing**

**What is present:**

- `src/coverage/traces.rs` — `detect_trace_config_changes()` (glob-matched via `globset`) and `detect_trace_env_vars()` (with exclude pattern support). 9 unit tests covering OTEL/Jaeger/DataDog detection, invalid globs, env-var exclusion.
- `src/coverage/sdk.rs` — `detect_sdk_changes()` across Added/Removed/Modified states, case-insensitive pattern matching, git-show for previous content. 7 unit tests covering Rust/Python/JS/Go imports and case insensitivity.
- `src/config/model.rs:411` — `CoverageConfig` containing `TracesConfig` and `SdkConfig`. All defaults match plan (`otel*.yaml`, `jaeger*.yaml`, `datadog*.yaml`; `OTEL_*`, `JAEGER_*`, `DD_*`, `OTLP_*`; exclude `OTEL_SDK_DISABLED`).
- `src/commands/impact.rs:2003` — `enrich_trace_configs()` and `enrich_sdk_dependencies()` wired as Phase 1 enrichment hooks, guarded by `config.enabled && config.traces.enabled`.
- `src/impact/packet.rs:584` — `trace_config_drift: Vec<TraceConfigChange>`, `trace_env_vars: Vec<TraceEnvVarChange>`, `sdk_dependencies_delta: Option<SdkDependencyDelta>` all present.
- `src/impact/analysis.rs:409` — Risk scoring: trace config drift +3/file (cap 9), trace env vars +2/var (cap 8), SDK new +5/dep (cap 20), SDK modified +2/dep (cap 10).
- `src/impact/packet.rs:700` — `finalize()` sorts `trace_config_drift`, `trace_env_vars`, and `sdk_dependencies_delta` sub-vecs.
- `src/impact/packet.rs:786` — `truncate_for_context()` clears `trace_config_drift`, `trace_env_vars`, `sdk_dependencies_delta`.

**Gaps:**

- **DEV7-1 (High):** `src/output/human.rs` — `print_impact_summary()` has no sections for `trace_config_drift`, `trace_env_vars`, or `sdk_dependencies_delta`. Users get no human-readable output for M7-1 findings.
- **DEV7-2 (Low):** No `analyze_risk()` unit tests that exercise M7-1 risk scoring paths (trace_config_drift weight, trace_env_var weight, SDK added/modified weights). The logic exists but has zero test coverage.

---

### M7-2: Service-Map Derivation

**Status: SUBSTANTIALLY IMPLEMENTED — human output missing, two structural weaknesses**

**What is present:**

- `src/coverage/services.rs` — `infer_services()` groups routes by directory depth cap (3 levels), falls back to package.json/Cargo.toml names. `compute_cross_service_edges()` maps handler/model names to services and collapses duplicate edges. 8 unit tests.
- `src/commands/impact.rs:1867` — `populate_service_map()` queries service assignments via `project_files.service_name`, loads cross-service edges from structural_edges using `COALESCE(qualified_name, symbol_name)`, uses `old_path` for rename detection.
- `src/commands/impact.rs:228` — Guards on `config.coverage.enabled && config.coverage.services.enabled`.
- `src/impact/packet.rs:582` — `service_map_delta: Option<ServiceMapDelta>`.
- `src/impact/analysis.rs:463` — Risk: 2 affected services = +3, 3–4 = +8, 5+ = +15.
- `src/impact/analysis.rs:609` — `analyze_risk()` uses `extend` not overwrite, so service-map reasons survive.
- `src/impact/packet.rs:791` — `truncate_for_context()` clears `service_map_delta`.
- `src/index/project_index.rs:1328` — Service assignment uses depth-sorted services with `AND service_name IS NULL` guard to prevent root-level overwrites of already-assigned files.

**Gaps:**

- **DEV7-3 (High):** `src/output/human.rs` — no section for `service_map_delta`. Users see no service-map output.
- **DEV7-4 (Medium):** `src/coverage/services.rs:21` — `infer_services()` accepts `_call_graph: &CallGraph` and `_topology: &DirectoryTopology` but neither is used. Plan specifies call-graph topology should inform service boundary detection; currently only directory prefix grouping is used.
- **DEV7-5 (Medium):** `src/index/project_index.rs:1342` — Root-level services (empty or `.` directory) still produce `LIKE '%'` patterns. The depth-sort mitigation prevents overwriting files already assigned by deeper services, but any file not covered by a deeper service gets stamped by the root service. Repos with a mix of root handlers and sub-service handlers will still produce ambiguous assignments.

---

### M7-3: Data-Flow Coupling

**Status: SUBSTANTIALLY IMPLEMENTED — human output missing, sparse tests**

**What is present:**

- `src/coverage/dataflow.rs` — `compute_data_flow_coupling()` traverses call chains, checks ≥20% changed nodes and at least one data model in chain. Assigns High risk for chains >5 nodes or ≥3 changed nodes, otherwise Medium. 2 unit tests.
- `src/index/call_graph.rs:79` — `CallGraph::enumerate_call_chains()` implemented — DFS from route handlers up to `max_depth`, produces `Vec<CallChain>`, handles cycles.
- `src/commands/impact.rs:2027` — `enrich_data_flow()` loads routes, call edges, data models from DB; builds CallGraph; enumerates chains; computes coupling.
- `src/impact/packet.rs:580` — `data_flow_matches: Vec<DataFlowMatch>`.
- `src/impact/analysis.rs:481` — Risk: +4 per data-flow match (cap 20).
- `src/impact/packet.rs:700` — `finalize()` sorts `data_flow_matches`.
- `src/impact/packet.rs:786` — `truncate_for_context()` clears `data_flow_matches`.

**Gaps:**

- **DEV7-6 (High):** `src/output/human.rs` — no section for `data_flow_matches`.
- **DEV7-7 (Low):** `src/coverage/dataflow.rs` — only 2 tests (basic coupling and threshold check). Missing: chain length > 5 triggers High risk; empty chains skipped; multiple overlapping chains; cycle handling; no-data-model chains skipped.
- **DEV7-8 (Low):** No `analyze_risk()` tests exercising the data_flow_matches risk path.

---

### M7-4: Deployment Manifest Awareness

**Status: SUBSTANTIALLY IMPLEMENTED — human output missing, config weights not consumed**

**What is present:**

- `src/coverage/deploy.rs` — `detect_deploy_manifest_changes()` with glob matching; `classify_manifest()` for Dockerfile/DockerCompose/Kubernetes/Terraform/Helm. 2 unit tests.
- `src/config/model.rs:560` — `DeployConfig` with `patterns`, `risk_weight_per_manifest` (default 3), `risk_cap` (default 15).
- `src/commands/impact.rs:2137` — `enrich_deploy_manifests()` wired and guarded.
- `src/impact/packet.rs:590` — `deploy_manifest_changes: Vec<DeployManifestChange>`.
- `src/impact/analysis.rs:494` — Risk: +3/manifest (cap 15).
- `src/impact/packet.rs:708` — `finalize()` sorts `deploy_manifest_changes`.
- `src/impact/packet.rs:790` — `truncate_for_context()` clears `deploy_manifest_changes`.

**Gaps:**

- **DEV7-9 (High):** `src/output/human.rs` — no section for `deploy_manifest_changes`.
- **DEV7-10 (Medium):** `src/impact/analysis.rs:495` — Risk weights are hardcoded (`weight_per_manifest = 3`, `cap = 15`) rather than reading from `config.deploy.risk_weight_per_manifest` and `config.deploy.risk_cap`. The config fields exist but are unused by `analyze_risk()`, which does not receive `CoverageConfig` as a parameter.
- **DEV7-11 (Low):** `src/coverage/deploy.rs` — only 2 tests. Missing: deleted manifests, Helm classification, invalid glob patterns.

---

### M7-5: CI Self-Awareness

**Status: PARTIALLY IMPLEMENTED — enrich hook is a stub, config weights not consumed**

**What is present:**

- `src/config/model.rs:599` — `CiSelfAwarenessConfig` with `enabled`, `ci_changed_weight` (default 3), `ci_plus_source_weight` (default 5).
- `src/commands/impact.rs:192` — `populate_ci_gates()` wired (populates `ChangedFile.ci_gates` from DB, pre-existing M5 feature).
- `src/impact/analysis.rs:507` — Risk logic: if any changed file has CI gates populated, add +5 if source code also changed, +3 if CI only.
- `src/impact/analysis.rs:609` — Reasons appended via `extend` (correct).

**Gaps:**

- **DEV7-12 (High):** `src/output/human.rs` — no section surfacing CI pipeline impact.
- **DEV7-13 (High):** `src/commands/impact.rs:2152` — `enrich_ci_self_awareness()` is a no-op stub. The comment says "CI self-awareness logic is partially implemented in analyze_risk() using existing packet.changes[].ci_gates." The guard check on `config.ci_self_awareness.enabled` exits immediately but does nothing else. The CI gate detection relies on `populate_ci_gates()` which runs unconditionally regardless of `config.ci_self_awareness.enabled`.
- **DEV7-14 (Medium):** `src/impact/analysis.rs:511` — Weights are hardcoded (5/3) instead of reading from `config.ci_self_awareness.ci_plus_source_weight` and `config.ci_self_awareness.ci_changed_weight`. `analyze_risk()` does not receive `CoverageConfig`.
- **DEV7-15 (Low):** The existing `test_analyze_risk_ci_gates` tests (in analysis.rs) test the pre-M7 CI gate behavior; there are no tests that validate the M7-5 config-guard behavior or the distinction between CI-only and CI+source weight paths.

---

### M7-6: ADR Staleness

**Status: SUBSTANTIALLY IMPLEMENTED — analyze_risk threshold and staleness source deviate from plan**

**What is present:**

- `src/impact/packet.rs:253` — `staleness_days: Option<u32>` on `RelevantDecision`.
- `src/config/model.rs:627` — `AdrStalenessConfig` with `enabled` and `threshold_days` (default 365).
- `src/commands/impact.rs:2166` — `enrich_adr_staleness()` iterates `packet.relevant_decisions` and populates `staleness_days` from `std::fs::metadata().modified().elapsed()`.
- `src/impact/analysis.rs:517` — Adds stale-ADR reasons for decisions with `staleness_days > 365`.

**Gaps:**

- **DEV7-16 (High):** `src/output/human.rs` — no section for stale decisions.
- **DEV7-17 (Medium):** `src/impact/analysis.rs:520` — Threshold hardcoded as `days > 365` instead of reading `config.adr_staleness.threshold_days`. If a user sets `threshold_days = 180`, the advisory still fires at 365.
- **DEV7-18 (Medium):** Plan (`docs/observability-plan2.md`) specifies staleness_days should be populated in `src/retrieval/query.rs` during retrieval so it is available from the start of packet construction. Instead, it is populated as a post-enrichment step via filesystem mtime in `enrich_adr_staleness()`. For files referenced by relative path (e.g. `docs/adr/0001.md`), `std::fs::metadata()` resolves relative to the process working directory, which may differ from the repo root in some invocation contexts.
- **DEV7-19 (Low):** No tests for `enrich_adr_staleness()` or for the ADR staleness advisory path in `analyze_risk()`.

---

### M7-7: Packet Extension

**Status: CORE COMPLETE — all human output sections missing**

**What is present:**

- `src/impact/packet.rs` — All five new fields present: `data_flow_matches`, `service_map_delta`, `trace_config_drift`, `trace_env_vars`, `sdk_dependencies_delta`, `deploy_manifest_changes`.
- `src/commands/impact.rs:209` — All seven enrichment hooks invoked in `execute_impact` with independent `warn` fallback on error (graceful degradation).
- `src/impact/packet.rs:700` — `finalize()` sorts: `data_flow_matches` (by change_pct desc), `trace_config_drift` (Ord impl), `trace_env_vars` (Ord impl), `sdk_dependencies_delta.*` sub-vecs (Ord impl), `deploy_manifest_changes` (Ord impl).
- `src/impact/packet.rs:782` — `truncate_for_context()` clears all six M7 fields (`data_flow_matches`, `trace_config_drift`, `trace_env_vars`, `sdk_dependencies_delta`, `deploy_manifest_changes`, `service_map_delta`).

**Gaps:**

- **DEV7-20 (Critical):** `src/output/human.rs` — `print_impact_summary()` has zero coverage of any M7 field. All seven sections specified in the plan are missing:
  1. Trace config drift section
  2. Trace env var section  
  3. SDK dependency delta section
  4. Service map delta section
  5. Data-flow coupling section
  6. Deploy manifest changes section
  7. Stale ADR advisory section

  The `print_impact_summary()` function ends at observability signals, temporal couplings, affected contracts, and file analysis warnings. No M7 data is ever printed to the user.

---

## Cross-Cutting Findings

### DEV7-21 (Critical): All M7 human output sections absent

Every M7 track requires a new section in `print_impact_summary()`. None exist. This means M7 data is computed, scored into risk_reasons, serialized to JSON (`--output json`), but never shown in the default human terminal output. Users relying on `changeguard impact` without `--output json` see none of the M7 enrichment.

Files to update: `src/output/human.rs`.

### DEV7-22 (Medium): analyze_risk() cannot read CoverageConfig

`analyze_risk(packet, rules)` does not receive `CoverageConfig` as a parameter. This means all config-tunable M7 weights are hardcoded:
- SDK `risk_weight_new = 5`, `risk_weight_modified = 2` (match defaults, so harmless when defaults unchanged)
- Deploy `risk_weight_per_manifest = 3`, `risk_cap = 15` (match defaults)
- CI `ci_changed_weight = 3`, `ci_plus_source_weight = 5` (match defaults)
- ADR threshold `> 365` (hardcoded, ignores `adr_staleness.threshold_days`)

Config customization is silently ignored. Signature is `analyze_risk(packet: &mut ImpactPacket, rules: &Rules)` at `src/impact/analysis.rs:46`.

### DEV7-23 (Medium): enrich_ci_self_awareness is a no-op stub

`src/commands/impact.rs:2152` — The function checks the config toggle and immediately returns `Ok(())`. It adds no enrichment beyond what `populate_ci_gates()` already does unconditionally. The intended M7-5 behavior (gating CI gate detection on `coverage.ci_self_awareness.enabled`) is not implemented; `populate_ci_gates()` runs regardless of the config flag.

### DEV7-24 (Medium): infer_services() ignores call_graph and topology

`src/coverage/services.rs:22-23` — Parameters `_call_graph: &CallGraph` and `_topology: &DirectoryTopology` are accepted but never used. Service boundaries are inferred only from route source paths. Plan requires topology and call-graph to inform service grouping (e.g. workers, background processors without HTTP routes would remain invisible). Config option `coverage.services.enabled` controls the index-time step but not the inference quality.

### DEV7-25 (Low): Sparse unit test coverage for M7 detection modules

| Module | Tests | Missing coverage |
|--------|-------|-----------------|
| `coverage/traces.rs` | 9 | — adequate |
| `coverage/sdk.rs` | 7 | — adequate |
| `coverage/services.rs` | 8 | integration test for service → analyze_risk roundtrip |
| `coverage/dataflow.rs` | 2 | High-risk chain (>5 nodes), cycle handling, no-data-model skip |
| `coverage/deploy.rs` | 2 | deleted manifests, Helm, invalid globs |
| `analysis.rs` (M7 paths) | 0 | All M7 risk-scoring paths (4a–4h) untested |
| `enrich_adr_staleness` | 0 | Function has no tests at all |

---

## Resolved Prior Findings (from audit6.md memory)

The previous audit session recorded HIGH findings for M7-2 that appear in the in-memory summary. Current code status:

| Prior Finding | Status in Current Code |
|--------------|----------------------|
| Service-map risk signals overwritten by analyze_risk() | **FIXED** — `analyze_risk()` uses `extend` at line 609, not `=` |
| Renamed files excluded from affected-service detection | **FIXED** — `populate_service_map()` uses `old_path.as_ref().unwrap_or(&path)` at line 1882 |
| Cross-service edges use bare symbol_name | **IMPROVED** — now uses `COALESCE(qualified_name, symbol_name)` in both populate_service_map and enrich_data_flow queries |
| Root-level LIKE '%' service claims whole repo | **MITIGATED** — depth-sort + `AND service_name IS NULL` guard prevents deeper services from being overwritten, but root LIKE '%' still fires for unassigned files |
| service_map_delta not cleared in truncate_for_context | **FIXED** — cleared at packet.rs line 791 |
| infer_services() ignores _call_graph and _topology | **STILL OPEN** (DEV7-24) |

---

## Finding Priority Summary

| ID | Severity | Track | Summary |
|----|----------|-------|---------|
| DEV7-20 | Critical | M7-7 | All 7 human output sections missing from print_impact_summary() |
| DEV7-21 | Critical | Cross | M7 data invisible in default terminal output |
| DEV7-1 | High | M7-1 | No human output for trace_config_drift / trace_env_vars / sdk_dependencies_delta |
| DEV7-3 | High | M7-2 | No human output for service_map_delta |
| DEV7-6 | High | M7-3 | No human output for data_flow_matches |
| DEV7-9 | High | M7-4 | No human output for deploy_manifest_changes |
| DEV7-12 | High | M7-5 | No human output for CI gate impact |
| DEV7-13 | High | M7-5 | enrich_ci_self_awareness is a no-op stub |
| DEV7-16 | High | M7-6 | No human output for stale ADR advisories |
| DEV7-22 | Medium | Cross | analyze_risk() cannot read CoverageConfig — weights hardcoded |
| DEV7-23 | Medium | M7-5 | populate_ci_gates runs unconditionally regardless of ci_self_awareness.enabled |
| DEV7-24 | Medium | M7-2 | infer_services() ignores _call_graph and _topology params |
| DEV7-4 | Medium | M7-2 | Same as DEV7-24 |
| DEV7-5 | Medium | M7-2 | Root-level LIKE '%' service assignment still present (mitigated but not fixed) |
| DEV7-10 | Medium | M7-4 | Config risk weights not consumed by analyze_risk |
| DEV7-14 | Medium | M7-5 | CI weights hardcoded in analyze_risk |
| DEV7-17 | Medium | M7-6 | ADR threshold hardcoded (ignores threshold_days config) |
| DEV7-18 | Medium | M7-6 | staleness_days populated from mtime not retrieval pipeline |
| DEV7-2 | Low | M7-1 | No analyze_risk() tests for M7-1 risk paths |
| DEV7-7 | Low | M7-3 | Only 2 tests in dataflow.rs |
| DEV7-8 | Low | M7-3 | No analyze_risk() tests for M7-3 risk paths |
| DEV7-11 | Low | M7-4 | Only 2 tests in deploy.rs |
| DEV7-15 | Low | M7-5 | No M7-5-specific analyze_risk tests |
| DEV7-19 | Low | M7-6 | No tests for enrich_adr_staleness or ADR advisory path |
| DEV7-25 | Low | Cross | Sparse test coverage across M7 detection modules |

---

## Recommended Fix Order

1. **DEV7-20/21** (Critical): Implement all 7 human output sections in `print_impact_summary()`.
2. **DEV7-13/23** (High): Fix `enrich_ci_self_awareness` to actually gate `populate_ci_gates` on the config toggle, or document the intentional unconditional behavior.
3. **DEV7-22** (Medium): Add `CoverageConfig` parameter to `analyze_risk()` so configured weights are respected.
4. **DEV7-17** (Medium): Use `config.adr_staleness.threshold_days` in `analyze_risk()` instead of `365`.
5. **DEV7-24** (Medium): Implement call-graph and topology usage in `infer_services()`.
6. **DEV7-18** (Medium): Populate `staleness_days` during retrieval (`query.rs`) rather than enrichment (`enrich_adr_staleness`).
7. **DEV7-7/11/19/25** (Low): Add tests for dataflow edge cases, deploy manifest types, ADR staleness, and M7 risk scoring paths.

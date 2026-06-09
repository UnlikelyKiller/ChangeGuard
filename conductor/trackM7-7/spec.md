# Specification: Track M7-7 — Impact Packet Extension & Enrichment Integration

## Objective
Wire all M7 detection signals into `ImpactPacket`, risk scoring, human output, and ask context. Implement master kill switch and per-dimension toggles. Ensure all new fields follow the determinism contract.

## Components

### 1. CoverageConfig (`src/config/model.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoverageConfig {
    pub enabled: bool,
    pub traces: TracesCoverageConfig,
    pub sdk: SdkCoverageConfig,
    pub services: ServicesCoverageConfig,
    pub data_flow: DataFlowCoverageConfig,
    pub deploy: DeployCoverageConfig,
    pub ci_self_awareness: CiSelfAwarenessConfig,
    pub adr_staleness: AdrStalenessConfig,
}

// Each sub-config has its own `enabled: bool` field defaulting to false
```

### 2. ImpactPacket Fields

```rust
// M7-1
pub trace_config_drift: Vec<TraceConfigChange>,
pub sdk_dependencies_delta: SdkDependencyDelta,

// M7-2
pub service_map_delta: Option<ServiceMapDelta>,

// M7-3
pub data_flow_matches: Vec<DataFlowMatch>,

// M7-4
pub deploy_manifest_changes: Vec<DeployManifestChange>,

// M7-5 — no new fields (risk_reasons only)

// M7-6 — staleness_days + staleness_tier added to existing RelevantDecision
```

### 3. Determinism Contract

All `Vec` fields:
- Element types implement `Eq + Ord`
- Sorted in `finalize()`: by primary sort key descending
- Cleared in `truncate_for_context()` Phase 3
- `SdkDependencyDelta` sorts its inner `added`/`modified`/`removed` Vecs

### 4. Enrichment Hooks (`src/commands/impact.rs`)

```rust
// After analyze_risk() (following audit5 fix pattern):

if config.coverage.enabled {
    enrich_trace_configs(&mut packet, &config.coverage.traces);
    enrich_sdk_dependencies(&mut packet, &config.coverage.sdk);
    enrich_service_map(&mut packet, &config.coverage.services, conn);
    enrich_data_flow(&mut packet, &config.coverage.data_flow, conn);
    enrich_deploy_manifests(&mut packet, &config.coverage.deploy);
    enrich_ci_self_awareness(&mut packet, &config.coverage.ci_self_awareness);
    enrich_adr_staleness(&mut packet, &config.coverage.adr_staleness, conn);
}
```

Each hook checks its per-dimension `enabled` flag independently. The master `coverage.enabled` flag gates all.

### 5. Risk Weighting

| Signal | Weight per unit | Cap |
|---|---|---|
| Trace config file changed | 3 | 9 |
| Trace env var changed | 2 | 8 |
| New SDK dependency | 5 | 20 |
| Modified SDK dependency | 2 | 10 |
| Cross-service change | 3-15 (by count) | — |
| Data-flow chain match | 4 | 20 |
| Deploy manifest changed | 3-8 (by type) | 15 |
| CI config changed alone | 3 | — |
| CI config + source changed | 5 | — |
| Pre-commit hook changed | 2 | — |

### 6. Human Output

Seven new sections in `src/output/human.rs`, each conditionally rendered when its field is non-empty:
- **Observability Config Drift** — trace config files and env vars
- **SDK Dependencies** — added/modified/removed SDKs
- **Service Map** — affected services and cross-service edges
- **Data-Flow Coupling** — call chains with co-changed nodes
- **Deployment Manifests** — Dockerfile, k8s, terraform, helm changes
- **CI Pipeline Impact** — CI config changed warnings
- **ADR Staleness** — staleness tier annotations

### 7. Ask Context

New enrichment sections injected into ask context with budget enforcement. If context is near 38k token limit, M7 sections are dropped as lowest priority after M6 sections.

## Test Specifications

| Test | Assertion |
|---|---|
| Master kill switch `coverage.enabled = false` | Zero M7 fields populated, risk score unchanged |
| Per-dimension toggle `traces.enabled = false` | `trace_config_drift` empty, other fields still populated |
| All new Vec fields sorted in finalize | Correct sort order verified per field |
| All new Vec fields cleared in truncate | Fields empty after Phase 3 truncation |
| Serialization roundtrip all fields populated | JSON → deserialize → serialize matches |
| Serialization roundtrip all fields empty | Empty fields absent from JSON |
| Risk weight caps enforced | 100 trace configs → weight capped at 9 |
| Human output conditional rendering | Empty field → section not printed |
| Human output all sections populated | All 7 sections rendered correctly |
| Ask context budget enforcement | M7 sections dropped before M6 when budget exhausted |
| Config backward compatibility | M6-era config loaded by M7 binary → identical behavior |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **Master kill switch**: `[coverage].enabled = false` must be a true zero-impact toggle.
- **Enrichment after analyze_risk**: Following audit5 fix pattern, all enrichment runs after `analyze_risk()`.
- **No hot-path embedding**: Verify no `embed_long_text`/`embed_batch` call exists in any M7 hook.
- **Config backward compat**: Existing configs deserialize identically when `[coverage]` is absent.

## Hardening Additions (in plan)

| Addition | Reason |
|---|---|
| Master kill switch | Single toggle to disable all M7 behavior |
| Per-dimension kill switches | Independent toggles allow gradual opt-in |
| Enrichment after analyze_risk | Prevents risk reason overwrite (audit5 pattern) |
| Risk weight caps | Prevents single-signal domination of risk score |
| Determinism contract for all new Vec fields | Byte-identical impact reports |
| Serialization roundtrip all scenarios | No data loss in JSON pipeline |
| Human output conditional rendering | Clean output when features not configured |
| Ask context budget enforcement | Respects existing 38k token budget |
| No hot-path embedding verification | Guarantees no embedding calls in impact path |
| Config backward compatibility | M6 configs work identically with M7 binary |

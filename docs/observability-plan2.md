# Milestone M7: Engineering Coverage Deepening

## Overview

This plan continues from Milestone M6 (M6-2) to address the remaining engineering onboarding dimensions ChangeGuard can naturally cover — without becoming a monitoring platform, CI system, or secret manager.

The four dimensions (Observability, API & Integration, Architecture & Design, Process & Workflow) were assessed against ChangeGuard's existing index and impact pipeline. The gaps that fit are:

1. **Observability Stack** — Trace/config awareness (not trace ingestion)
2. **API & Integration** — Third-party SDK dependency, service-to-route mapping
3. **Architecture & Design** — Data-flow coupling, ADR staleness
4. **Process & Workflow** — Deployment manifest impact, CI pipeline self-awareness

These are index-time widening or risk-weight additions — no new external services, no new SQLite tables (with one possible exception for trace configs), no new HTTP clients. The embedding pipeline from Milestone M is reused where semantic matching applies.

---

## 0. Executive Summary

Milestone M gave ChangeGuard the ability to index documents, query live observability, match API contracts, and predict test failures — all powered by the local embedding pipeline.

Milestone M7 adds **breadth** to what ChangeGuard detects during `scan` and weights during `impact`:

- **M7-1 — Trace Config & SDK Detection**: Flag observability pipeline config changes and new third-party SDK imports
- **M7-2 — Service-Map Derivation**: Infer service boundaries from route/handler/data-model topology
- **M7-3 — Data-Flow Coupling**: Flag call chains where route handlers and their data models co-change
- **M7-4 — Deployment Manifest Awareness**: Weight Dockerfiles, k8s manifests, and IaC into risk scoring
- **M7-5 — CI Pipeline Self-Awareness**: Surface risk when CI config itself changes in a diff
- **M7-6 — ADR Staleness Detection**: Flag retrieved ADRs exceeding a configurable age threshold
- **M7-7 — Impact Packet Extension & Enrichment**: Wire all new fields into `ImpactPacket`, `finalize()`, `truncate_for_context()`, risk scoring, and ask context

All additions are additive and degrade gracefully. No Milestone M behavior is altered.

---

## 1. Product Intent

ChangeGuard remains a **local-first change intelligence and verification orchestration CLI**. This expansion deepens its input surface so it can answer:

- "Did I touch the observability config?"
- "Did I add a new payment-gateway dependency?"
- "Is the route I changed coupled to a data model that also changed?"
- "Does the changed deployment manifest risk a production outage?"
- "Are the ADRs that matched my change stale?"

These signals flow into the existing `ImpactPacket.risk_reasons`, `risk_level`, and `ask` context. No new commands. No new output formats.

---

## 2. Core Implementation Principles

### 2.1 Non-Negotiable Principles

All principles from Milestone M §2.1 carry forward. Additional:

1. **No new infrastructure.** These features use the existing scanner, indexer, and risk scorer. At most one new SQLite table (trace configs). No new HTTP clients, no new storage backends.
2. **Index-time, not hot-path.** All new detection happens during `scan` / `changeguard index`, not during `impact`. Impact enriches from pre-indexed data.
3. **Config-driven.** Each new dimension is individually toggleable via `changeguard.toml`. Unconfigured dimensions are silent no-ops.
4. **Deterministic output.** All new `Vec` fields on `ImpactPacket` are sorted in `finalize()` and cleared in `truncate_for_context()` (following existing pattern from M2-2, M5-2, M6-2).

---

## 3. Architecture Boundaries

### 3.1 New Module

```
src/coverage/
  mod.rs            — module declarations, shared types
  traces.rs         — trace-config file + env-var detection (M7-1)
  sdk.rs            — third-party SDK import detection (M7-1)
  services.rs       — service-map derivation from routes + topology (M7-2)
  dataflow.rs       — data-flow coupling detection (M7-3)
  deploy.rs         — deployment manifest classification (M7-4)
```

All symbols are `pub(crate)`. Consumed by `scan`, `impact`, and `ask` commands.

### 3.2 Extensions to Existing Modules

| Module | Track | Change |
|--------|-------|--------|
| `src/index/routes.rs` | M7-2 | Add `infer_service()` — group routes by directory topology |
| `src/index/call_graph.rs` | M7-3 | Add `compute_data_flow_coupling()` — flag route→handler→model chains |
| `src/index/ci_gates.rs` | M7-5 | Add self-awareness risk reason |
| `src/index/project_index.rs` | M7-4 | Detect Dockerfile, k8s YAML, terraform as deploy manifests |
| `src/index/env_schema.rs` | M7-1 | Flag `OTEL_*`, `JAEGER_*`, `DD_*`, `OTLP_*` env vars |
| `src/retrieval/query.rs` | M7-6 | Add `stale_threshold_days` — flag ADRs older than threshold |
| `src/impact/packet.rs` | M7-7 | Add 5 new `Vec` fields + `SdkDependencyDelta` |
| `src/impact/analysis.rs` | M7-7 | Wire new risk weights into `analyze_risk()` |
| `src/commands/ask.rs` | M7-7 | Inject new enrichment fields into ask context |
| `src/output/human.rs` | M7-7 | Render new enrichment sections in human output |
| `src/config/model.rs` | M7-7 | Add `CoverageConfig` with sub-sections |

---

## 4. Track M7-1: Trace Config & SDK Dependency Detection

**Dependencies**: M5-2 (observability pattern), M1-2 (embed for any semantic matching)

### 4.1 Trace Config File Detection

During `scan`, extend the file classifier to recognize trace configuration files:

| File pattern | Signal |
|---|---|
| `*otel-collector*.yaml`, `*otel-collector*.yml` | OpenTelemetry Collector config |
| `*jaeger-agent*.yaml`, `*jaeger-agent*.yml` | Jaeger agent config |
| `*datadog-agent*.yaml`, `*datadog.yaml` | DataDog agent config |
| `*grafana-agent*.yaml` | Grafana Agent config |
| `*tempo*.yaml` | Grafana Tempo config |

Risk reason: "Observability pipeline configuration modified — traces/metrics collection may be affected ({file})."

### 4.2 Trace Env-Var Detection

Extend `src/index/env_schema.rs` to flag trace-related env vars:

| Pattern | Signal |
|---|---|
| `OTEL_*` | OpenTelemetry SDK/collector |
| `JAEGER_*` | Jaeger client |
| `DD_*` | DataDog client |
| `OTLP_*` | OTLP endpoint |

Risk reason: "Observability environment variable changed: {var_name}"

### 4.3 Third-Party SDK Detection

During `scan`, the language-aware symbol extractor already collects imports. Extend it to match against a configurable SDK pattern list:

```toml
[sdk_detection]
patterns = [
    "stripe", "auth0", "twilio", "aws-sdk", "@aws-sdk",
    "google-cloud", "sendgrid", "firebase", "supabase",
    "openai", "anthropic",
]
```

- **New SDK added**: Medium elevation. "New third-party SDK introduced: {sdk} in {file}"
- **SDK modified**: Low elevation. "Third-party SDK integration modified: {sdk} in {file}"
- **SDK removed**: Informational only

### 4.4 Key Deliverables

| File | Content |
|------|---------|
| `src/coverage/mod.rs` | Module declarations |
| `src/coverage/traces.rs` | `detect_trace_config_changes()`, `detect_trace_env_vars()` |
| `src/coverage/sdk.rs` | `detect_sdk_changes()` |
| Tests | Tempfile fixtures for trace configs, import scanning |

### 4.5 Hardening Additions

| Addition | Reason |
|---|---|
| Case-insensitive SDK matching | `Stripe`, `STRIPE`, and `stripe` in imports must all match. Use `.to_lowercase()` on both the import text and the pattern. |
| Language-aware import extraction | Python `from stripe import Charge`, Rust `use stripe::Charge`, JS `import { Stripe } from 'stripe'`, Go `import "github.com/stripe/stripe-go"` — each needs the import path extracted before matching. |
| Glob-safe config pattern validation | Invalid glob patterns in `config_patterns` must log `WARN` and skip, never abort `scan`. |
| Double-extension trace files | `otel-collector.yaml.tmpl`, `jaeger-agent.yml.dist` — config patterns should use `*` suffix matching rather than exact extension matching. |
| Stale trace config detection | If a trace config file existed in the previous scan but has been deleted, record it as removed (not just "no longer present"). |
| `--no-trace` env-var exclusion | Allow users to configure `exclude_env_patterns` at the project level to suppress noisy trace env var matches (e.g., `OTEL_SDK_DISABLED`). |

---

## 5. Track M7-2: Service-Map Derivation

**Dependencies**: M6-1 (routes), existing call graph + topology

### 5.1 Inference Pipeline

The existing route extraction (`src/index/routes.rs`), call graph (`src/index/call_graph.rs`), and directory topology (`src/index/topology.rs`) are combined to infer a service map:

1. Group routes by the top-level directory containing their handler file (e.g., `src/api/users/` → service "users")
2. Each service: `{ name: String, routes: Vec<RouteRef>, data_models: Vec<ModelRef> }`
3. Cross-service edges from call-graph edges where caller and callee are in different services

### 5.2 Risk Impact

| Changed services | Elevation | Reason |
|---|---|---|
| 2 | Low | Cross-service change: {svc_a} → {svc_b} |
| 3-4 | Medium | Multi-service change spanning {n} services |
| 5+ | High | Large blast radius: {n} services affected |

### 5.3 Storage

Computed at index-time. Stored in the existing `symbol_centrality` or a new derived table. Impact queries pre-computed service assignments — no hot-path computation.

### 5.4 Key Deliverables

| File | Content |
|------|---------|
| `src/coverage/services.rs` | `infer_services()`, `compute_cross_service_edges()` |
| `src/index/routes.rs` (extend) | `infer_service_for_route()` |
| Tests | Multi-file repo fixture, route → service grouping |

### 5.5 Hardening Additions

| Addition | Reason |
|---|---|
| Multi-strategy service naming | Fallback chain: directory name → package name (Rust `Cargo.toml`, Python `__init__.py`, JS `package.json` nearest parent) → "unnamed-service-N". Never produce empty service names. |
| Service deduplication by route ownership | If two directories both contain routes to `/users`, the service owning the closest handler file to the route definition wins. Document the tiebreak. |
| Monorepo-aware directory depth | Flat repos (`src/handler.rs` → service "src") should produce one service, not explode. Cap detection depth at 2 levels. |
| Empty route set fallback | Projects without detected routes (CLI tools, libraries) return `service_map_delta: None` rather than an empty map. |
| Cross-service edge dedup | If service A has 5 call-graph edges to service B, collapse them into one cross-service edge with an edge count. |

---

## 6. Track M7-3: Data-Flow Coupling Risk

**Dependencies**: M7-2 (service map), existing data models + call graph

### 6.1 Detection

The call graph contains call chains. The data model index contains struct/class definitions per file. Combine:

```
Route /users/{id} → handler get_user() → calls UserRepo.find_by_id()
                                      → returns User struct
```

Flag when changed files contain both a route handler AND the data model it touches. Flag when multiple files in the same call chain change together.

### 6.2 Risk Impact

- **Route + data model co-changed**: Medium elevation. "Data-flow coupling: {route} handler and {model} model changed together."
- **Call chain with 3+ changed nodes**: Medium elevation per node. "Call chain affected: {chain}"
- **Chain depth > 5**: High elevation regardless of changed count.

### 6.3 Key Deliverables

| File | Content |
|------|---------|
| `src/coverage/dataflow.rs` | `compute_data_flow_coupling()`, `DataFlowMatch` type |
| `src/index/call_graph.rs` (extend) | `enumerate_call_chains()` |
| Tests | Fixture with route→handler→repo→model chain, co-change detection |

### 6.4 Hardening Additions

| Addition | Reason |
|---|---|
| Cycle detection | Call graphs with cycles (A→B→A) must terminate after `chain_depth_max` iterations, not loop infinitely. |
| Changed-node percentage threshold | If a call chain has 20 nodes and only 1 changed, it's not coupling. Require ≥20% of chain nodes to be changed to flag. |
| External-node filtering | A call to `std::fs::read` or `println!` should not create a data-flow chain. Filter out standard-library / framework nodes. |
| Database model resolution | If a handler calls `db.query("SELECT * FROM users")` without an explicit `User` struct reference, use the SQL table name as the model name. Fallback to "unknown-model". |
| Chain depth floor | Chains shorter than 2 nodes (route → handler only) are not flagged. Minimum chain depth is 2. |

---

## 7. Track M7-4: Deployment Manifest Awareness

**Dependencies**: None (pure file classifier)

### 7.1 Detection

During `scan`, classify files matching deployment patterns:

| Pattern | ManifestType |
|---|---|
| `Dockerfile`, `Dockerfile.*` | `Dockerfile` |
| `docker-compose*.yml`, `docker-compose*.yaml` | `DockerCompose` |
| `*.tf`, `*.tfvars` | `Terraform` |
| `k8s/**/*.yaml`, `kubernetes/**/*.yaml` | `Kubernetes` |
| `helm/**/*.yaml`, `Chart.yaml` | `Helm` |
| `.github/workflows/*.yml` | `CiWorkflow` |

### 7.2 Risk Impact

| Change | Elevation | Reason |
|---|---|---|
| Dockerfile changed | Low | Container build configuration modified |
| Docker Compose / K8s / Helm changed | Medium | Container orchestration modified |
| Terraform changed | Medium | Infrastructure-as-code modified — provisioned resources may change |
| 2+ manifest types in same diff | High | Deployment surface changed across {n} manifest types |

Risk weight: 3 per manifest file, cumulated with a cap at 15.

### 7.3 Key Deliverables

| File | Content |
|------|---------|
| `src/coverage/deploy.rs` | `classify_deploy_manifest()`, `ManifestType` enum |
| `src/index/project_index.rs` (extend) | Integration into file classification |
| Tests | Tempfile fixtures with Dockerfile, docker-compose, k8s, terraform |

### 7.4 Hardening Additions

| Addition | Reason |
|---|---|
| Dockerfile instruction scanning | Detect `COPY`, `ADD`, and `FROM` directives. If `COPY src/ ./src/` in Dockerfile and `src/` changed, escalate risk by one tier. |
| docker-compose service → file coupling | If `docker-compose.yml` defines `build: ./api` and `./api/Dockerfile` also changed, escalate from Low to Medium. |
| Terraform plan/output drill-down | Parse Terraform files for `resource`, `module`, `variable` blocks. Flag when a resource type with high blast radius (`aws_rds_cluster`, `kubernetes_deployment`) is touched. |
| Helm values coupling | If `Chart.yaml` + `values.yaml` change together, add a specific coupling reason. |
| Skip binary files in manifest directories | `k8s/**/*.yaml` should not match `.yaml.bak` or binary files with `.yaml` extension. Validate YAML parseability before classifying. |
| Multi-manifest dedup | If both `Dockerfile` and `Dockerfile.prod` change, count as 1 Dockerfile manifest type with a count annotation, not 2 separate risk reasons. |

---

## 8. Track M7-5: CI Pipeline Self-Awareness

**Dependencies**: Existing `src/index/ci_gates.rs`

### 8.1 Detection

The existing CI gate detector (`src/index/ci_gates.rs`) already discovers and parses CI configuration files. Extend it to add a risk reason when CI config itself appears in the change set:

- **CI config changed alone**: Low elevation. "CI pipeline configuration modified — build or test gates may have changed ({file})"
- **CI config changed alongside source code**: Medium elevation. "CI pipeline and application code changed in same diff — gates may behave unexpectedly for the new code."

### 8.2 Implementation

No new packet fields. Append to `risk_reasons` with weight 3 (CI-only) or 5 (CI+source). Detection happens during `impact` by checking if any changed file path matches a known CI config pattern.

### 8.3 Key Deliverables

| File | Content |
|------|---------|
| `src/index/ci_gates.rs` (extend) | `is_ci_config_file()`, risk reason injection |
| Tests | Diffs containing CI config changes |

### 8.4 Hardening Additions

| Addition | Reason |
|---|---|
| Non-standard CI detection | If no known CI config files are detected but `.github/`, `.gitlab-ci.yml`, or `Jenkinsfile` appear in changed files, flag as "Unknown CI pipeline file changed". |
| CI + deploy coupling | If CI config AND deploy manifests change in the same diff, the CI self-awareness risk and deploy risk compound (not add — escalate by one tier). |
| Skip generated CI files | `.github/workflows/generated-*.yml` or files with `# auto-generated` headers should not trigger CI awareness. |
| Pre-commit hook detection | If `.pre-commit-config.yaml` or `lefthook.yml` changes, flag as "Pre-commit hooks modified — local checks may change". |

---

## 9. Track M7-6: ADR Staleness Detection

**Dependencies**: M2-2 (retrieval / RelevantDecision)

### 9.1 Detection

The existing `src/retrieval/query.rs` retrieves relevant documentation by semantic similarity. Extend it to check the age of matched ADRs:

```toml
[docs]
stale_threshold_days = 365
```

When a retrieved ADR exceeds the threshold: "Warning: matched ADR '{title}' is {age_days} days old — may not reflect current architecture."

### 9.2 Risk Impact

This is a **warning**, not a risk reason. It appears in the `ask` context and human impact output but does not change `risk_level`. Age is computed from the ADR's file modification time (or a date metadata field if present).

### 9.3 Key Deliverables

| File | Content |
|------|---------|
| `src/retrieval/query.rs` (extend) | `compute_staleness()`, `stale_threshold_days` config |
| `src/impact/packet.rs` (extend) | `staleness_days: Option<u32>` on `RelevantDecision` |
| Tests | Tempfile docs with known mtimes, threshold filtering |

### 9.4 Hardening Additions

| Addition | Reason |
|---|---|
| Multi-source age detection | Check ADR file mtime, then ADR frontmatter `date:` field, then `created:` metadata line. Use the most recent date found. |
| Recently-updated ADRs exempt | If an ADR's mtime is within 30 days, never flag it as stale regardless of its creation date — it was recently reviewed. |
| Staleness severity tiers | < 365 days: no flag. 365-730 days: "may need review". > 730 days: "significantly stale — may not reflect current architecture". |
| Staleness documentation in ask context | When an ADR is flagged stale, the ask context should include a brief explanation of why staleness matters for this change, not just a date. |
| Git-based age for unversioned docs | If a document has no date metadata and no filesystem mtime (e.g., generated content), fall back to `git log --follow` to find the last modification date. |

---

## 10. Track M7-7: Impact Packet Extension & Enrichment

**Dependencies**: M7-1 through M7-6

### 10.1 New ImpactPacket Fields

```rust
pub struct ImpactPacket {
    // --- existing fields from M1-M6 ---

    // M7-1: Trace + SDK
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trace_config_drift: Vec<TraceConfigChange>,
    #[serde(default)]
    pub sdk_dependencies_delta: SdkDependencyDelta,

    // M7-2: Service map
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_map_delta: Option<ServiceMapDelta>,

    // M7-3: Data-flow
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_flow_matches: Vec<DataFlowMatch>,

    // M7-4: Deploy
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deploy_manifest_changes: Vec<DeployManifestChange>,

    // M7-5: CI self-awareness — no new fields (risk_reasons only)

    // M7-6: staleness_days added to existing RelevantDecision field
}
```

All new `Vec` fields sorted in `finalize()` by primary key descending, cleared in `truncate_for_context()` Phase 3.

### 10.2 Risk Weighting

| Signal | Weight | Cap |
|---|---|---|
| Trace config file changed | 3 per file | 9 |
| Trace env var changed | 2 per var | 8 |
| New SDK dependency | 5 per SDK | 20 |
| Modified SDK dependency | 2 per SDK | 10 |
| Cross-service change (2 svcs) | 3 | — |
| Cross-service change (3-4 svcs) | 8 | — |
| Cross-service change (5+ svcs) | 15 | — |
| Data-flow chain match | 4 per chain | 20 |
| Deploy manifest changed | 3 per manifest | 15 |
| CI config changed alone | 3 | — |
| CI config + source changed | 5 | — |

### 10.3 Enrichment Hooks

Following the existing enrichment pattern from M2-2, M5-2, M6-2:

```rust
// In src/commands/impact.rs:
enrich_trace_configs(&mut packet, &config);
enrich_sdk_dependencies(&mut packet, &config);
enrich_service_map(&mut packet, &config, conn);
enrich_data_flow(&mut packet, &config, conn);
enrich_deploy_manifests(&mut packet, &config);
enrich_ci_self_awareness(&mut packet, &config);
enrich_adr_staleness(&mut packet, &config, conn);

// Then analyze_risk...
// Then escalate_risk from enrichment signals...
```

### 10.4 Human Output

Extend `src/output/human.rs` with new sections (all empty → not shown):

- **Observability Config Drift** — trace config files and env vars changed
- **SDK Dependencies** — new/modified/removed third-party SDK imports
- **Service Map** — affected services and cross-service edges
- **Data-Flow Coupling** — call chains with co-changed nodes
- **Deployment Manifests** — changed Dockerfiles, k8s, terraform, helm
- **CI Pipeline Impact** — CI config changed warning
- **ADR Staleness** — warnings for stale matched documents

### 10.5 Key Deliverables

| File | Content |
|------|---------|
| `src/config/model.rs` | `CoverageConfig` with sub-sections for each dimension |
| `src/impact/packet.rs` | 5 new types + 5 new fields + finalize/truncate |
| `src/impact/analysis.rs` | Risk weight wiring for all new signals |
| `src/commands/impact.rs` | 7 enrichment hooks |
| `src/commands/ask.rs` | Context injection for new enrichment fields |
| `src/output/human.rs` | 7 new output sections |
| Tests | 20+ tests for enrichment, ordering, serialization, GDP |

### 10.6 Hardening Additions

| Addition | Reason |
|---|---|
| Master kill switch | `[coverage].enabled = false` must disable ALL enrichment hooks, all new detection during scan, and all ask context injection. Test: with enabled=false, packet has zero M7 fields and risk_score is identical to pre-M7. |
| Per-dimension kill switches | Each `[coverage.*].enabled` sub-toggle must independently disable its dimension without affecting others. Test matrix validates combinations. |
| Enrichment hook ordering | Hooks must run after `analyze_risk()` (matching audit5 fix pattern). Enrichment results must not be overwritten by risk analysis. |
| Risk weight caps enforcement | Test each signal type hits its cap and stops contributing weight. Verify that 100 trace config changes contributes only 9 weight, not 300. |
| Determinism contract for all new Vec fields | Every new `Vec` field must have `Ord` on its element type, sorted in `finalize()` by primary key descending, and cleared in `truncate_for_context()` Phase 3. Includes: `trace_config_drift`, `data_flow_matches`, `deploy_manifest_changes`. `SdkDependencyDelta` sorts its inner `added`/`modified`/`removed` Vecs. |
| Serialization roundtrip | Every new packet field must survive `serde_json::to_string → from_str` without loss. Test with all fields populated, all fields empty, and mixed. |
| Human output conditional rendering | Each of 7 new sections must NOT render when its source field is empty. Must render with correct column alignment when populated. |
| Ask context budget enforcement | New enrichment sections injected into ask context must respect the existing 38k token budget. If context is near the budget limit, M7 sections are dropped after all M6 sections (lowest priority). |
| No hot-path embedding | Verify no `embed_long_text` or `embed_batch` call exists in any M7 enrichment hook. All detection is file-classification or pre-indexed queries. |
| Config backward compatibility | Adding `[coverage]` section to config must not change deserialization of existing sections. Test: load an M6-era config with a post-M7 binary — all M6 behavior identical. |

---

## 11. New Configuration Sections

```toml
[coverage]
enabled = false                    # master toggle for M7

[coverage.traces]
enabled = true
config_patterns = ["**/otel*.yaml", "**/jaeger*.yaml", "**/datadog*.yaml"]
env_var_patterns = ["OTEL_*", "JAEGER_*", "DD_*", "OTLP_*"]

[coverage.sdk]
enabled = true
patterns = ["stripe", "auth0", "twilio", "sendgrid", "openai", "anthropic"]
risk_weight_new = 5
risk_weight_modified = 2

[coverage.services]
enabled = true
cross_service_elevation_threshold = 2

[coverage.data_flow]
enabled = true
chain_depth_max = 5

[coverage.deploy]
enabled = true
patterns = ["**/Dockerfile*", "**/docker-compose*.yml", "**/*.tf", "**/k8s/**/*.yaml"]
risk_weight_per_manifest = 3
risk_cap = 15

[coverage.ci_self_awareness]
enabled = true
ci_changed_weight = 3
ci_plus_source_weight = 5

[coverage.adr_staleness]
enabled = true
threshold_days = 365
```

All `enabled` default to `false`. The top-level `[coverage].enabled = false` disables all M7 enrichment at once. No breaking changes to existing configuration.

---

## 12. Delivery Sequence

| Order | Track | Depends On | Fires During |
|---|---|---|---|
| 1 | M7-1 — Trace + SDK Detection | M1-2, M5-2 (patterns) | `scan` |
| 2 | M7-4 — Deployment Manifest | None (file classifier) | `scan` |
| 3 | M7-2 — Service-Map Derivation | M6-1 (routes), call graph | `index` |
| 4 | M7-3 — Data-Flow Coupling | M7-2, data models | `impact` |
| 5 | M7-5 — CI Self-Awareness | Existing ci_gates.rs | `impact` |
| 6 | M7-6 — ADR Staleness | M2-2 (retrieval) | `impact` |
| 7 | M7-7 — Packet Extension + Enrichment | M7-1..M7-6 | `impact` |

M7-1 and M7-4 can be implemented in parallel (no shared module surface). M7-2 + M7-3 are sequential. M7-5 and M7-6 are independent. M7-7 is the final integration track, identical in role to M2-2/M5-2/M6-2.

---

## 13. Testing Strategy

Following the Milestone M pattern:

| Track | Test Count (est.) | Key Test Fixture |
|---|---|---|
| M7-1 | 12 | Tempfile trace configs, source files with SDK imports |
| M7-2 | 10 | Multi-file repo with routes, handlers, models across dirs |
| M7-3 | 8 | Call-chain fixture with co-changed files |
| M7-4 | 10 | Tempfile Docker, k8s, terraform manifests |
| M7-5 | 6 | Diffs with CI config changes |
| M7-6 | 8 | Tempfile ADRs with known mtimes |
| M7-7 | 20+ | Full packet roundtrip, enrichment ordering, serialization |

**Total**: ~74 new tests. All existing 604 tests MUST pass with `[coverage].enabled = false`.

---

## 14. New Dependency Additions

None. All M7 detection uses existing dependencies:
- File I/O: `std::fs` (already used)
- Pattern matching: `regex` (already in Cargo.toml)
- AST extraction: existing language modules in `src/index/languages/`
- Config: `toml` / `serde` (already in Cargo.toml)

---

## 15. What This Plan Deliberately Excludes

- **Real-time trace ingestion** — not a code-change risk tool's domain
- **Diagram rendering** — presentation concern
- **Secrets management** — HashiCorp Vault / AWS Secrets Manager belongs in deployment tooling
- **New HTTP clients** — all detection is AST-level or file-classifier, not runtime
- **New databases** — only one possible SQLite table for trace config snapshots; otherwise the existing 5 M tables + ledger tables suffice
- **Notification / alerting** — platform concern (PagerDuty, Slack webhooks)
- **CI/CD orchestration** — deployment execution belongs in CI runners, not a CLI tool
- **Git-flow enforcement** — team process, not change risk

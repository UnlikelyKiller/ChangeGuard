# Track W10 Spec: Runtime Observability, SLO, and Alert Ownership Graph

## Background

Runtime and observability signal tracking currently scores 6/10. ChangeGuard has Prometheus/log/pattern support and observability impact enrichment, but SLOs, alerts, dashboards, incidents, owners, and runtime identity links are weak.

## Objective

Raise observability tracking to 9/10 by modeling metrics, logs, traces, alerts, dashboards, SLOs, incidents, owners, and service links as graph entities.

## Proposed Design

1. Add observability graph nodes for metric, log pattern, trace span, alert, dashboard, SLO, incident, runtime service, and owner.
2. Parse common alerting, dashboard, and SLO config files locally.
3. Link runtime signals to services, endpoints, code symbols, deploy manifests, tests, ADRs, and incidents.
4. Add impact rules for changed code with missing observability, changed SLO-owned services, alerts without owners, and deploy changes without runtime signal coverage.
5. Add `changeguard observability diff` and `changeguard observability coverage`.

## Critical Files

| File | Expected work |
|---|---|
| `src/observability/prometheus.rs` | Keep live query optional and local-first |
| `src/observability/log_scanner.rs` | Link log patterns to graph nodes |
| `src/impact/enrichment/observability.rs` | Add SLO, alert, and owner risk rules |
| `src/coverage/traces.rs` | Link trace config to services and endpoints |
| `src/commands/` and `src/cli.rs` | Add observability review commands |

## Definition of Done

- Runtime signals link to services, endpoints, deploy surfaces, tests, and owners where known.
- Missing observability on high-risk changes is reported with remediation guidance.
- Live Prometheus or incident integrations remain optional.
- Target score after completion: 9/10.

# Track W10 Plan: Runtime Observability, SLO, and Alert Ownership Graph

- [ ] Task W10.1: Create fixtures for metrics, log patterns, traces, dashboards, alerts, SLOs, and service ownership.
- [ ] Task W10.2: Write tests for observability graph nodes and service/endpoint links.
- [ ] Task W10.3: Add parsers for local alert, dashboard, and SLO config formats.
- [ ] Task W10.4: Link runtime signal nodes to services, endpoints, deploy manifests, tests, ADRs, and owners.
- [ ] Task W10.5: Add impact rules for missing observability, alert owner gaps, and SLO-owned service changes.
- [ ] Task W10.6: Implement `changeguard observability diff` and `changeguard observability coverage`.
- [ ] Task W10.7: Add local-first docs explaining optional live signal ingestion.
- [ ] Task W10.8: Run observability, coverage, impact, and full verification gates; reinstall.

## Definition of Done Checklist

- [ ] Observability coverage is inspectable per service and endpoint.
- [ ] SLO and alert owner gaps are explicit.
- [ ] Live observability integrations are optional, not required for tests.
- [ ] Full verification gate passes.

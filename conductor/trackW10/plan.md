# Track W10 Plan: Runtime Observability, SLO, and Alert Ownership Graph

- [ ] Task W10.1: Create fixtures for OpenSLO `Service`, `SLO`, `SLI`, `DataSource`, `AlertPolicy`, metrics, log patterns, traces, dashboards, alerts, and service ownership.
- [ ] Task W10.2: Write tests for observability graph nodes and service/endpoint links.
- [ ] Task W10.3: Add the primary OpenSLO YAML importer with schema-version checks and deterministic object linking.
- [ ] Task W10.4: Add secondary parsers for local alert, dashboard, and provider-specific SLO config formats only where they provide evidence not available through OpenSLO.
- [ ] Task W10.5: Link runtime signal nodes and OpenSLO targets to services, endpoints, deploy manifests, tests, ADRs, and owners.
- [ ] Task W10.6: Add impact rules for missing observability, alert owner gaps, OpenSLO target changes, and SLO-owned service changes.
- [ ] Task W10.7: Implement `changeguard observability diff` and `changeguard observability coverage`.
- [ ] Task W10.8: Add local-first docs explaining OpenSLO ingestion and optional live signal ingestion.
- [ ] Task W10.9: Run observability, coverage, impact, and full verification gates; reinstall.

## Definition of Done Checklist

- [ ] Observability coverage is inspectable per service and endpoint.
- [ ] OpenSLO objects are linked to service, endpoint, alert, and owner graph nodes where known.
- [ ] SLO and alert owner gaps are explicit.
- [ ] Live observability integrations are optional, not required for tests.
- [ ] Full verification gate passes.

# ChangeGuard Tracking Ability

This document scores how well ChangeGuard currently tracks the main surfaces that matter in a large repository, then outlines changes that would bring each category as close to a 10/10 as is reasonable while preserving ChangeGuard's local-first, CLI-first architecture.

Scores use this scale:

- 1-3: minimal or incidental support
- 4-6: partial support with meaningful gaps
- 7-8: useful support for real large-repo workflows
- 9-10: strong support with durable data model, impact integration, verification hooks, and query/reporting affordances

The target graph shape should bias toward:

```text
endpoint -> handler -> symbol -> test -> ADR -> ledger_tx -> service -> data/config/runtime signal
```

That graph gives impact analysis enough structure to answer "what breaks if I change this?" without requiring a centralized cloud platform.

## Summary Scores

> Last updated: v0.1.3, post Milestone W (W1–W13 complete).

| Tracking area | Current score | Reasonable target score | Current support |
|---|---:|---:|---|
| Public APIs and endpoints | 9 | 9 | Endpoint graph with auth/schema/consumer links; `endpoints --json/--changed` |
| ADRs and decisions | 8 | 9 | Entity-link tables, `ledger graph`, validator lifecycle; lifecycle transitions still implicit |
| Service boundaries | 9 | 9 | Declared service map, queue/topic edges, `services diff` |
| Data models and migrations | 9 | 9 | Durable relations, compatibility classification, `data-models impact --changed` |
| Config and env vars | 9 | 9 | Schema metadata, `config schema/diff`; live secret inventory remains external |
| CI/CD and deployment surfaces | 8 | 9 | Deploy surface graph nodes and Cedar cross-links; promotion/rollback paths not modeled |
| Dependency and SDK usage | 8 | 9 | Lockfile ingestion, advisory matching; full transitive graph coverage incomplete |
| Tests and verification mapping | 8 | 9 | Durable test nodes, `verify explain --entity`; dynamic/runtime coverage inference limited |
| Runtime/observability signals | 8 | 9 | SLO/metric/alert nodes, OpenSLO parsing, `observability diff/coverage`; live incident state external |
| Hotspots and temporal coupling | 10 | 10 | Trend snapshots, `hotspots trend/explain`, ownership links, policy thresholds |
| Provenance/ledger transactions | 10 | 10 | Validator lifecycle, entity links, `ledger graph`, hook-repair rollback |
| Security boundaries | 9 | 9 | Cedar parsing, cross-surface links, `security boundaries/impact`; app-specific authz semantics remain |

Overall current score: 8.9/10.

Reasonable target after the changes below: 9.2/10.

## Public APIs and Endpoints

Current score: 9/10. *(was 7 pre-W)*

Current support:

- Rust route extraction records method, path pattern, handler symbol, framework, source, confidence, and evidence.
- Contract indexing parses OpenAPI and Swagger specs into endpoint records.
- Contract matching can flag public contract risk in impact packets.
- Auth extraction for common Rust middleware and annotations with config-driven hints.
- Request/response schema links via `api_endpoint_contract` relation to handlers and data models.
- Consumer mapping from generated clients and HTTP callsites stored as durable graph nodes.
- Impact rules for auth changes, removed endpoints, path/method changes, and consumer-visible behavior.
- `changeguard endpoints --json` and `changeguard endpoints --changed` for direct review.

Remaining gaps:

- Full consumer ownership is often organizational data outside the repo. ChangeGuard ingests what is visible but cannot infer external consumer registries.

Reasonable target score: 9/10.

## ADRs and Decisions

Current score: 8/10. *(was 6 pre-W)*

Current support:

- Ledger entries can generate MADR-formatted ADRs.
- Ledger search provides full-text decision lookup.
- ADR staleness can be surfaced through coverage and retrieval flows.
- ADR nodes stored in the graph, linked to ledger transactions via entity-link tables.
- `ledger graph <tx-id>` shows the entity neighborhood governed by a transaction, including linked ADRs.
- Security-scoped ADR linking via label keyword matching (Cedar policy `governs` ADR edges).

Remaining gaps:

- Structured decision fields (owner, supersedes, superseded_by, reviewed_at) are not fully enforced as schema.
- `ledger adr update-status` / `ledger adr review` commands for lifecycle transitions are not yet implemented.
- Stale/contradictory ADR detection when governed entities are modified is not yet automatic.
- Human decision quality cannot be guaranteed by tooling.

Reasonable target score: 9/10.

## Service Boundaries

Current score: 9/10. *(was 7 pre-W)*

Current support:

- `[services]` config declares service roots, owners, runtime names, queues, topics, APIs, and contacts.
- Cross-service edge kinds: HTTP, RPC, queue, event, database, config, shared library, and generated client.
- Service nodes linked to deployment manifests, CI jobs, endpoints, data models, ADRs, and owner metadata.
- `changeguard services diff` shows service boundary changes in a PR-style view.

Remaining gaps:

- Service ownership and runtime names still require explicit team-maintained metadata; cannot be fully inferred.

Reasonable target score: 9/10.

## Data Models and Migrations

Current score: 9/10. *(was 7 pre-W)*

Current support:

- Durable relations for data_model, table, column, migration, schema_change, and compatibility_class.
- Migration parsing with classification of add/drop/rename/type-change/index/backfill operations.
- ORM models and SQL table usage linked to endpoints, services, tests, and migrations.
- Risk rules for destructive migrations, missing backfill tests, incompatible schema changes, and unowned data models.
- `changeguard data-models impact --changed` for focused data review.

Remaining gaps:

- Framework coverage is necessarily incomplete. SQL, ORM, and migration patterns vary widely across languages and stacks.

Reasonable target score: 9/10.

## Config and Environment Variables

Current score: 9/10. *(was 8 pre-W)*

Current support:

- Env schema extraction tracks declarations and references.
- References distinguish read, write, defaulted, and dynamic usage.
- Secret-like names and undeclared variables are surfaced.
- Explicit env var schema metadata: required, optional, default, secret, owner, environment, rotation policy, provider, and rollout notes.
- Env vars linked to services, endpoints, deploy manifests, tests, ADRs, and runtime incidents.
- Detection for config key removals, default changes, secret exposure, environment-only config drift, and missing examples.
- `changeguard config schema` and `changeguard config diff` with JSON output.
- Policy support for protected or required env vars per service.

Remaining gaps:

- External secret manager truth. ChangeGuard can model and verify repo-visible expectations, but live secret inventory is external.

Reasonable target score: 9/10.

## CI/CD and Deployment Surfaces

Current score: 8/10. *(was 7 pre-W)*

Current support:

- CI self-awareness detects workflow/config changes.
- Deployment coverage detects Dockerfile, Compose, Kubernetes, Terraform, and Helm surfaces.
- Deployment graph nodes for environment, artifact, service, manifest, workflow, job, secret, and owner.
- Deploy manifests linked to services, runtime config, endpoints, and observability signals.
- Cedar cross-surface links connect policies to deploy surfaces.
- Risk rules for changed release gates, removed tests/checks, changed deployment strategy, and unowned infrastructure.
- `changeguard deploy impact --changed` and `changeguard ci diff`.

Remaining gaps:

- Promotion paths, rollback plans, and live deployment artifact identity are not modeled.
- Live production inventory belongs behind an optional integration boundary, not local repo analysis.

Reasonable target score: 9/10.

## Dependency and SDK Usage

Current score: 8/10. *(was 6 pre-W)*

Current support:

- Dependency graph ingestion for Cargo, npm, Python, and lockfiles.
- Package, version, source, license, direct/transitive edge, owning service, and known advisory fields stored as durable relations.
- Provider-specific SDK use extraction linked to endpoints, services, env vars, and auth/config requirements.
- Advisory scanner adapters consume cargo-audit and OSV output without cloud services.
- Impact rules for vulnerable dependency introduction, major-version upgrades, removed SDKs, and provider auth/config changes.

Remaining gaps:

- Full transitive graph coverage across all package manager ecosystems is incomplete; v1 targets the most common cases.

Reasonable target score: 9/10.

## Tests and Verification Mapping

Current score: 8/10. *(was 7 pre-W)*

Current support:

- Durable test nodes with test kind, owner, target entity, risk class, flakiness, last result, and coverage confidence.
- Tests linked to endpoints, handlers, symbols, data models, migrations, config keys, and services.
- Predictive verification uses structural calls, temporal coupling, runtime dependencies, and test mappings.
- `changeguard verify explain --entity <path>` for entity-scoped test explanation and mapping.
- Risk rules for high-impact changes without mapped tests or with stale/flaky mapped tests.

Remaining gaps:

- Coverage import adapters for external coverage formats (lcov, cobertura) are not yet implemented.
- Dynamic/runtime coverage cannot be perfectly proven through static inference.

Reasonable target score: 9/10.

## Runtime and Observability Signals

Current score: 8/10. *(was 6 pre-W)*

Current support:

- Prometheus client, log scanner, and observability signal model exist.
- Observability graph nodes for metric, log pattern, trace span, alert, and SLO.
- OpenSLO YAML parsing with source-file metadata injected into graph nodes for reliable diff matching.
- Runtime signals linked to services, endpoints, code symbols, deploy manifests, and tests.
- Impact rules for changed code with missing observability, changed SLO-owned services, and alerts without owners.
- `changeguard observability diff` and `changeguard observability coverage`.

Remaining gaps:

- Dashboard and incident nodes are not modeled.
- Live incident and alert state is optional integration data, not a hard local dependency.

Reasonable target score: 9/10.

## Hotspots and Temporal Coupling

Current score: 10/10. *(was 9 pre-W)*

Current support:

- Hotspot ranking combines churn and complexity.
- Temporal coupling detects co-change relationships.
- Hotspots and temporal coupling feed output, impact, verification, and bridge export.
- Persistent hotspot and temporal snapshots over time support trend deltas.
- Owner/service/test links to hotspots.
- Hotspot budgets and policy thresholds per directory or service.
- `changeguard hotspots trend`, `hotspots explain`, and `hotspots budget`.
- Remediation suggestions tied to actual test and ownership graph data.

Reasonable target score: 10/10.

## Provenance and Ledger Transactions

Current score: 10/10. *(was 9 pre-W)*

Current support:

- Transaction lifecycle, drift detection, reconciliation, adoption, rollback, atomic entries, FTS search, signing, federation, and ADR generation exist.
- Ledger transactions link to git commits.
- Commit validators and tech stack enforcement exist.
- Full validator lifecycle: `ledger validator list`, `disable`, `enable`, `remove`, and `doctor`.
- Entity-link tables from ledger transaction to symbol, endpoint, service, data model, config key, test, ADR, and deploy surface.
- Hook lifecycle diagnostics and repair commands for sidecar/pending mismatches (hook-repair rollback).
- `changeguard ledger graph <tx-id>` shows the entity neighborhood governed by a transaction.
- Provenance export with stable schema for audit and external ingestion.

Reasonable target score: 10/10.

## Security Boundaries

Current score: 9/10. *(was 7 pre-W)*

Current support:

- Secret redaction exists for impact and prompt surfaces.
- Protected paths exist in policy.
- Process policy guards verification execution.
- Ledger signing and signature verification exist.
- Commit validators can enforce local security checks.
- Cedar policy parsing with principal/action/resource graph nodes.
- Cross-surface heuristic linking: policy → endpoint, service, config_key, deploy_surface, ADR via `protected_by` and `governs` edges.
- `changeguard security boundaries` emits both auth nodes and the cross-surface boundary graph.
- `changeguard security impact --changed` for policy-scoped change analysis.

Remaining gaps:

- Authorization semantics are highly app-specific. Cross-surface links use heuristics (raw text substring matching); precise semantic inference is not always possible.

Reasonable target score: 9/10.

## Cross-Cutting Changes Needed

Several improvements would raise multiple categories at once:

1. Add typed graph relations for ownership, service, endpoint, data model, config key, test, ADR, deployment, dependency, observability signal, and ledger transaction.
2. Add entity-link tables in SQLite and CozoDB so every impact packet can explain both direct and transitive relationships.
3. Add config-driven metadata overlays for facts that cannot be inferred reliably, especially owners, service names, auth requirements, SLOs, and external consumers.
4. Add `diff` and `explain` commands for each major surface so large-repo users can inspect changed graph neighborhoods directly.
5. Add stable JSON schemas for all tracking categories to support AI-Brains, CI, and external review tools.
6. Add repair commands for stale index, validator, hook, and graph-link drift.
7. Keep external integrations optional. ChangeGuard should ingest exported files or local API responses, but local repo analysis must remain useful offline.

## Proposed Track Grouping

If this work is planned through conductor tracks, a practical breakdown is:

- Track A: Entity graph schema and typed relations.
- Track B: Endpoint/auth/consumer graph hardening.
- Track C: ADR lifecycle and decision-link model.
- Track D: Service/deploy/observability ownership overlays.
- Track E: Dependency/advisory graph ingestion.
- Track F: Test mapping confidence and coverage import.
- Track G: Ledger validator lifecycle and transaction graph UX.
- Track H: Security boundary extraction and policy risk rules.
- Track I: Surface-specific `diff`, `explain`, and JSON output commands.

This sequence strengthens the shared graph first, then layers domain extractors and user-facing review surfaces on top.

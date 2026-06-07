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

| Tracking area | Current score | Reasonable target score | Current support |
|---|---:|---:|---|
| Public APIs and endpoints | 7 | 9 | Good extraction and contract matching, incomplete ownership/auth/consumer graph |
| ADRs and decisions | 6 | 9 | Ledger-backed MADR generation and staleness, weak lifecycle metadata |
| Service boundaries | 7 | 9 | Useful inferred service model, weak ownership and async topology |
| Data models and migrations | 7 | 9 | Data model extraction and migration structure, limited compatibility semantics |
| Config and env vars | 8 | 9 | Strong declaration/reference tracking, limited ownership and requiredness |
| CI/CD and deployment surfaces | 7 | 9 | Good manifest/CI detection, limited environment and ownership metadata |
| Dependency and SDK usage | 6 | 9 | SDK deltas exist, transitive and advisory graph is shallow |
| Tests and verification mapping | 7 | 9 | Predictive verification and test mapping exist, confidence needs refinement |
| Runtime/observability signals | 6 | 9 | Prometheus/log/pattern support exists, weak SLO and alert ownership |
| Hotspots and temporal coupling | 9 | 10 | Strong core capability; target is operational polish and trend history |
| Provenance/ledger transactions | 9 | 10 | Strong lifecycle and signing; target is validator UX and richer links |
| Security boundaries | 7 | 9 | Protected paths, redaction, signing, process policy; weak authz graph |

Overall current score: 7.4/10.

Reasonable target after the changes below: 9.2/10.

## Public APIs and Endpoints

Current score: 7/10.

Current support:

- Rust route extraction records method, path pattern, handler symbol, framework, source, confidence, and evidence.
- Contract indexing parses OpenAPI and Swagger specs into endpoint records.
- Contract matching can flag public contract risk in impact packets.

Current gaps:

- Endpoint auth requirements are not first-class.
- Request/response schemas are mostly available only through indexed OpenAPI data, not linked deeply to handlers and models.
- Consumer mapping is limited. Downstream callers, clients, and external consumers are not represented as durable graph nodes.

Changes to reach the target:

- Add an `api_endpoint_contract` relation that links route extraction, OpenAPI operations, request schemas, response schemas, handler symbols, and service owner.
- Add auth extraction for common Rust middleware and annotations, plus config-driven auth hints for frameworks ChangeGuard cannot infer safely.
- Add client/consumer extraction from generated clients, HTTP callsites, OpenAPI imports, and sibling schema federation.
- Add impact rules for auth changes, response schema changes, removed endpoints, path/method changes, and consumer-visible behavior.
- Add `changeguard endpoints --json` and `changeguard endpoints --changed` for direct review.

Reasonable target score: 9/10.

The remaining gap to 10 is that full consumer ownership is often organizational data outside the repo. ChangeGuard should ingest it when available, not pretend it can infer all of it.

## ADRs and Decisions

Current score: 6/10.

Current support:

- Ledger entries can generate MADR-formatted ADRs.
- Ledger search provides full-text decision lookup.
- ADR staleness can be surfaced through coverage and retrieval flows.

Current gaps:

- ADR owner, status, superseded-by, supersedes, affected services, and reviewed-at metadata are not consistently modeled.
- Decisions are not strongly linked to the symbols, endpoints, services, and config they govern.
- ADR lifecycle transitions are implicit rather than enforced.

Changes to reach the target:

- Add structured decision fields to ledger entries: status, owner, reviewers, supersedes, superseded_by, affected_entities, decision_scope, reviewed_at.
- Store ADRs as graph nodes linked to ledger transactions, services, endpoints, modules, config keys, and data models.
- Add `ledger adr update-status`, `ledger adr link`, and `ledger adr review` commands instead of relying only on generated markdown.
- Add stale/contradictory ADR detection when code changes touch governed entities.
- Add impact warnings when a change violates an active ADR or touches an expired/unreviewed one.

Reasonable target score: 9/10.

The remaining gap to 10 is human decision quality. Tooling can enforce metadata and links, but it cannot guarantee that a decision is correct.

## Service Boundaries

Current score: 7/10.

Current support:

- Service inference can group routes and data models by topology and project file metadata.
- Service map deltas expose affected services and cross-service edges.
- Deployment and coverage providers can raise service-level blast radius.

Current gaps:

- Owning team/module metadata is not first-class.
- Queues, event topics, scheduled jobs, RPC clients, and external service calls are not modeled consistently.
- Service boundaries are inferred more than declared, which is useful but can be ambiguous in large monorepos.

Changes to reach the target:

- Add optional `service-map.toml` or `[services]` config that declares service roots, owners, runtime names, queues, topics, APIs, data stores, and contacts.
- Add extractors for queue/topic producers and consumers across common libraries.
- Add cross-service edge kinds: HTTP, RPC, queue, event, database, config, shared library, and generated client.
- Link service nodes to deployment manifests, CI jobs, endpoints, data models, ADRs, and owner metadata.
- Add `changeguard services diff` to show service boundary changes in a PR-style view.

Reasonable target score: 9/10.

The remaining gap to 10 is that service ownership and runtime names usually need explicit team-maintained metadata.

## Data Models and Migrations

Current score: 7/10.

Current support:

- Data model extraction exists and feeds data-flow coupling.
- State migrations are modular and versioned.
- Data-flow coupling can flag route/data-model co-change risk.

Current gaps:

- Backward compatibility risk is inferred indirectly, not modeled through schema contracts.
- Ownership and domain boundaries for data models are limited.
- Database migration impact is not fully linked to endpoints, services, tests, and historical incidents.

Changes to reach the target:

- Add durable relations for data_model, table, column, migration, schema_change, and compatibility_class.
- Parse common migration formats and classify add/drop/rename/type-change/index/backfill operations.
- Link ORM models and SQL table usage to endpoints, services, tests, and migrations.
- Add risk rules for destructive migrations, missing backfill tests, incompatible schema changes, and unowned data models.
- Add `changeguard data-models impact --changed` for focused data review.

Reasonable target score: 9/10.

The remaining gap to 10 is framework coverage. SQL, ORM, and migration patterns vary widely across languages and stacks.

## Config and Environment Variables

Current score: 8/10.

Current support:

- Env schema extraction tracks declarations and references.
- References distinguish read, write, defaulted, and dynamic usage.
- Secret-like names and undeclared variables are surfaced.
- Runtime usage deltas can contribute to impact.

Current gaps:

- Required versus optional semantics are not consistently inferred.
- Secret status is based on name/content heuristics, not provider metadata.
- Owner, environment scope, and rollout requirements are limited.

Changes to reach the target:

- Add explicit env var schema metadata: required, optional, default, secret, owner, environment, rotation policy, provider, and rollout notes.
- Link env vars to services, endpoints, deploy manifests, tests, ADRs, and runtime incidents.
- Add detection for config key removals, default changes, secret exposure, environment-only config drift, and missing examples.
- Add `config schema` and `config diff` commands with JSON output.
- Add policy support for protected or required env vars per service.

Reasonable target score: 9/10.

The remaining gap to 10 is external secret manager truth. ChangeGuard can model and verify repo-visible expectations, but live secret inventory is external.

## CI/CD and Deployment Surfaces

Current score: 7/10.

Current support:

- CI self-awareness detects workflow/config changes.
- Deployment coverage detects Dockerfile, Compose, Kubernetes, Terraform, and Helm surfaces.
- Impact can elevate risk for deployment and CI changes.

Current gaps:

- Release environments, deployment ownership, promotion paths, rollback plans, and deployed artifact identity are not modeled deeply.
- CI job to service/test ownership is limited.
- Infrastructure ownership is mostly inferred from paths and manifests.

Changes to reach the target:

- Add deployment graph nodes for environment, artifact, service, manifest, workflow, job, secret, and owner.
- Parse CI workflows into jobs, dependencies, triggers, required checks, artifacts, and deployment targets.
- Link deploy manifests to services, runtime config, endpoints, and observability signals.
- Add risk rules for changed release gates, removed tests/checks, changed deployment strategy, changed base images, and unowned infrastructure.
- Add `changeguard deploy impact --changed` and `changeguard ci diff`.

Reasonable target score: 9/10.

The remaining gap to 10 is live deployment state. Local repo analysis can model intent and configuration, but live production inventory belongs behind an optional integration boundary.

## Dependency and SDK Usage

Current score: 6/10.

Current support:

- SDK dependency detection tracks selected third-party providers.
- Dependency and vulnerability workflows are documented.
- Runtime and impact systems can flag relevant dependency changes.

Current gaps:

- Direct and transitive dependency graphs are not stored as first-class durable relations.
- Advisory ingestion and vulnerability matching are not a core indexed data source.
- Provider ownership and service-level dependency exposure are limited.

Changes to reach the target:

- Add dependency graph ingestion for Cargo, npm, Python, Go, and lockfiles where feasible.
- Store package, version, source, license, direct/transitive edge, owning service, and known advisory fields.
- Add provider-specific SDK use extraction linked to endpoints, services, env vars, and auth/config requirements.
- Add advisory scanner adapters that can consume cargo-deny/cargo-audit/npm audit/pip-audit output without requiring cloud services.
- Add impact rules for vulnerable dependency introduction, major-version upgrades, removed SDKs, and provider auth/config changes.

Reasonable target score: 9/10.

The remaining gap to 10 is ecosystem breadth. Maintaining perfect dependency semantics across every package manager is not reasonable for v1.

## Tests and Verification Mapping

Current score: 7/10.

Current support:

- Test mapping extraction exists.
- Predictive verification uses structural calls, temporal coupling, runtime dependencies, and test mappings.
- Semantic test outcome history is recorded when embeddings are available.

Current gaps:

- Symbol-level coverage confidence is not always precise.
- Test quality, risk class, and ownership are not modeled deeply.
- Missing-test recommendations are not strongly tied to endpoint/service/data/config graph nodes.

Changes to reach the target:

- Add durable test nodes with test kind, owner, target entity, risk class, flakiness, last result, and coverage confidence.
- Link tests to endpoints, handlers, symbols, data models, migrations, config keys, and services.
- Add coverage import adapters for common coverage formats where present.
- Add `verify explain --entity` and `tests for <entity>` query surfaces.
- Add risk rules for high-impact changes without mapped tests or with stale/flaky mapped tests.

Reasonable target score: 9/10.

The remaining gap to 10 is dynamic/runtime coverage. Static inference cannot perfectly prove behavioral coverage.

## Runtime and Observability Signals

Current score: 6/10.

Current support:

- Prometheus client, log scanner, and observability signal model exist.
- Observability patterns are indexed from code.
- Trace config and observability changes can affect impact.

Current gaps:

- SLOs, alerts, dashboards, on-call ownership, incidents, and service-level runtime identities are not first-class.
- Live signal ingestion is shallow and optional.
- Signals are not consistently linked back to endpoints, services, owners, and tests.

Changes to reach the target:

- Add observability graph nodes for metric, log pattern, trace span, alert, dashboard, SLO, incident, and owner.
- Parse common alerting/dashboard/SLO config files locally.
- Link runtime signals to services, endpoints, code symbols, deploy manifests, and tests.
- Add impact rules for changed code with missing observability, changed SLO-owned services, and alerts without owners.
- Add `observability diff` and `observability coverage` commands.

Reasonable target score: 9/10.

The remaining gap to 10 is live incident and alert state, which should be optional integration data rather than a hard local dependency.

## Hotspots and Temporal Coupling

Current score: 9/10.

Current support:

- Hotspot ranking combines churn and complexity.
- Temporal coupling detects co-change relationships.
- Hotspots and temporal coupling feed output, impact, verification, and bridge export.
- Recent work reduced noisy temporal output and improved query focus.

Current gaps:

- Historical trend storage and regression detection can be stronger.
- Ownership and remediation workflow for hotspots is limited.
- Hotspot health is not consistently linked to teams, tests, ADRs, and service risk.

Changes to reach the target:

- Persist hotspot and temporal snapshots over time to support trend deltas.
- Add owner/service/test links to hotspots.
- Add hotspot budgets and policy thresholds per directory or service.
- Add `hotspots trend`, `hotspots explain`, and `hotspots budget` commands.
- Add remediation suggestions tied to actual test and ownership graph data.

Reasonable target score: 10/10.

This is a realistic 10 because the domain is local and repo-native: git history, complexity, service links, and tests are all observable from the repository plus optional metadata.

## Provenance and Ledger Transactions

Current score: 9/10.

Current support:

- Transaction lifecycle, drift detection, reconciliation, adoption, rollback, atomic entries, FTS search, signing, federation, and ADR generation exist.
- Ledger transactions can link to git commits.
- Commit validators and tech stack enforcement exist.

Current gaps:

- Validator management UX is incomplete. Register exists, but disable/remove/list-by-id lifecycle is weak.
- Ledger entries should link more deeply to graph entities such as endpoints, services, tests, ADRs, config keys, and runtime signals.
- Hook-created pending transaction lifecycle still needs ergonomic cleanup in edge cases.

Changes to reach the target:

- Add validator IDs plus `ledger validator list`, `disable`, `enable`, `remove`, and `doctor`.
- Add entity-link tables from ledger transaction to symbol, endpoint, service, data model, config key, test, ADR, and deploy surface.
- Add hook lifecycle diagnostics and repair commands for sidecar/pending mismatches.
- Add `ledger graph <tx-id>` to show the entity neighborhood governed by a transaction.
- Add provenance export with stable schema for audit and external ingestion.

Reasonable target score: 10/10.

This is a realistic 10 because provenance is ChangeGuard's native domain and the remaining gaps are concrete UX/data-model work.

## Security Boundaries

Current score: 7/10.

Current support:

- Secret redaction exists for impact and prompt surfaces.
- Protected paths exist in policy.
- Process policy guards verification execution.
- Ledger signing and signature verification exist.
- Commit validators can enforce local security checks.

Current gaps:

- Endpoint auth and authorization models are not first-class.
- Permission boundaries, roles, scopes, and policy files are not consistently indexed.
- Threat-model ownership and security review requirements are not represented deeply.

Changes to reach the target:

- Add auth/authz graph nodes for middleware, route requirements, roles, scopes, policies, secret dependencies, and protected resources.
- Parse common auth patterns, IAM/policy files, and framework-specific permission annotations.
- Link security boundaries to endpoints, services, config keys, deploy manifests, ADRs, tests, and ledger transactions.
- Add risk rules for auth bypass, policy broadening, secret exposure, protected path edits, and missing security review.
- Add `security impact --changed` and `security boundaries` commands.

Reasonable target score: 9/10.

The remaining gap to 10 is that authorization semantics are highly app-specific. ChangeGuard should provide strong primitives and configurable extractors rather than claiming perfect inference.

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

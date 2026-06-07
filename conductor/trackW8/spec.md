# Track W8 Spec: Dependency, SDK, and Advisory Graph

## Background

Dependency and SDK usage tracking currently scores 6/10. ChangeGuard detects selected SDK deltas, but direct/transitive dependency graphs, advisory ingestion, provider ownership, and service exposure are shallow.

## Objective

Raise dependency tracking to 9/10 by ingesting package graphs and OSV advisory output locally, then linking dependencies and SDK usage to services, endpoints, config keys, auth, and risk.

## Proposed Design

1. Add dependency graph ingestion for Cargo, npm, Python, Go, and lockfiles where feasible.
2. Store package, version, source, license, direct/transitive edge, owning service, importing symbol, provider, and known advisory fields.
3. Add provider-specific SDK usage extraction linked to endpoints, services, env vars, auth/config requirements, and external calls.
4. Make OSV-Scanner JSON the primary advisory ingestion format, with support for offline runs such as `osv-scanner scan --offline --format json`.
5. Treat cargo-deny, cargo-audit, npm audit, and pip-audit JSON as optional compatibility imports, not the core data model.
6. Add impact rules for vulnerable dependency introduction, major upgrades, removed SDKs, provider auth/config changes, and OSV vulnerable-path evidence.

## Critical Files

| File | Expected work |
|---|---|
| `src/coverage/sdk.rs` | Extend SDK/provider extraction |
| OSV-Scanner JSON importer | Add versioned OSV result ingestion and fixture tests |
| `Cargo.toml` and lockfile parsers | Add local package graph adapters without network dependency |
| `src/impact/enrichment/` | Add dependency/advisory impact rules |
| `src/commands/` and `src/cli.rs` | Add dependency graph review output |
| `docs/` | Document OSV-Scanner offline workflow and optional compatibility imports |

## Definition of Done

- Direct and transitive dependency edges are queryable with service and import evidence where known.
- OSV-Scanner JSON is the primary advisory input and can be ingested from an offline local-database run.
- Compatibility imports do not force ChangeGuard to maintain four separate advisory schemas as first-class models.
- Impact output explains vulnerable paths and affected services without requiring live network calls.
- Target score after completion: 9/10.

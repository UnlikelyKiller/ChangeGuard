# Track W8 Spec: Dependency, SDK, and Advisory Graph

## Background

Dependency and SDK usage tracking currently scores 6/10. ChangeGuard detects selected SDK deltas, but direct/transitive dependency graphs, advisory ingestion, provider ownership, and service exposure are shallow.

## Objective

Raise dependency tracking to 9/10 by ingesting package graphs and advisory outputs locally, then linking dependencies and SDK usage to services, endpoints, config keys, auth, and risk.

## Proposed Design

1. Add dependency graph ingestion for Cargo, npm, Python, Go, and lockfiles where feasible.
2. Store package, version, source, license, direct/transitive edge, owning service, importing symbol, provider, and known advisory fields.
3. Add provider-specific SDK usage extraction linked to endpoints, services, env vars, auth/config requirements, and external calls.
4. Consume local scanner outputs such as cargo-deny, cargo-audit, npm audit, and pip-audit without requiring cloud services.
5. Add impact rules for vulnerable dependency introduction, major upgrades, removed SDKs, and provider auth/config changes.

## Critical Files

| File | Expected work |
|---|---|
| `src/coverage/sdk.rs` | Extend SDK/provider extraction |
| `Cargo.toml` and lockfile parsers | Add local package graph adapters without network dependency |
| `src/impact/enrichment/` | Add dependency/advisory impact rules |
| `src/commands/` and `src/cli.rs` | Add dependency graph review output |
| `docs/` | Document scanner output ingestion |

## Definition of Done

- Direct and transitive dependency edges are queryable with service and import evidence where known.
- Advisory data can be ingested from local scanner output and linked to package graph nodes.
- Impact output explains vulnerable paths and affected services without requiring live network calls.
- Target score after completion: 9/10.

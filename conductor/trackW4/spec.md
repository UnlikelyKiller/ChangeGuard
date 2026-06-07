# Track W4 Spec: Service Boundary Ownership and Async Topology

## Background

Service boundary tracking currently scores 7/10. ChangeGuard can infer service maps from routes and data models, but ownership and async topology such as queues, topics, scheduled jobs, RPC, and external calls are not modeled consistently.

## Objective

Raise service boundary tracking to 9/10 by combining inference with explicit service metadata overlays and first-class cross-service edge kinds.

## Proposed Design

1. Add optional `service-map.toml` or `[services]` config for service roots, owners, runtime names, queues, topics, APIs, data stores, contacts, and SLO references.
2. Add extractors for queue/topic producers and consumers across common libraries and configuration files.
3. Add edge kinds for HTTP, RPC, queue, event, database, config, shared library, and generated client dependencies.
4. Link service nodes to deployment manifests, CI jobs, endpoints, data models, ADRs, runtime signals, and owner metadata.
5. Add `changeguard services diff` for changed service boundary review.

## Critical Files

| File | Expected work |
|---|---|
| `src/coverage/services.rs` | Extend service derivation and edge kinds |
| `src/impact/enrichment/services.rs` | Add ownership and async topology risk |
| `src/config/model.rs` | Add service overlay config |
| `src/index/` | Add async topology extraction hooks |
| `src/commands/` and `src/cli.rs` | Add service diff command |

## Definition of Done

- Service graph contains owner, roots, runtime names, inbound/outbound endpoints, data stores, queues/topics, deploy surfaces, and known tests where available.
- Inferred facts and declared facts are merged with explicit conflict reporting.
- Async producer/consumer changes can raise impact risk and suggest verification.
- Target score after completion: 9/10.

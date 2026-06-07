# Track W1 Spec: Entity Graph Schema and Cross-Surface Links

## Background

The tracking score assessment in `docs/TrackingAbility.md` identifies the same structural gap across most categories: ChangeGuard extracts useful facts, but many facts are not linked as typed graph relations. Large-repo impact analysis needs stable paths such as `endpoint -> handler -> symbol -> test -> ADR -> ledger_tx -> service -> data/config/runtime signal`.

Current related support lives in CozoDB graph storage, impact packets, route extraction, service inference, data model extraction, env schema tracking, test mapping, ledger provenance, and deploy/observability enrichers.

## Objective

Create the shared graph foundation required by Tracks W2 through W13. This track should define durable node and edge schemas, stable IDs, migration strategy, query helpers, and JSON output contracts for cross-surface impact analysis.

## Tracking Areas Addressed

- Cross-cutting foundation for all twelve tracking areas.
- Enables target scores for endpoints, ADRs, service boundaries, data, config, CI/CD, dependencies, tests, observability, hotspots, ledger, and security.

## Proposed Design

1. Define first-class node kinds for symbol, endpoint, service, data model, migration, config key, deploy surface, CI job, dependency, test, observability signal, ADR, ledger transaction, hotspot, temporal coupling, and security boundary.
2. Define first-class edge kinds such as owns, handles, calls, covers, governs, supersedes, deploys, depends_on, emits, alerts_on, changed_with, validates, authenticates, authorizes, and touches_secret.
3. Add stable IDs that survive path normalization, Windows case differences, and metadata overlay merges.
4. Store links in CozoDB and mirror enough metadata in SQLite for ledger and offline audit commands.
5. Add graph-link query helpers so impact providers do not each hand-roll traversal logic.
6. Version every JSON shape used by downstream commands and bridge exports.

## Critical Files

| File | Expected work |
|---|---|
| `src/state/storage/cozo.rs` | Add or migrate graph relations and indexes |
| `src/state/migrations/` | Add migration files for any SQLite mirror tables |
| `src/impact/enrichment/kg_provider.rs` | Reuse typed relations for impact traversal |
| `src/impact/packet.rs` | Add versioned relation summaries without destabilizing old output |
| `src/bridge/export.rs` | Export graph links with stable schema |
| `src/index/` | Emit typed graph links from existing extractors |

## Definition of Done

- Existing extraction still works offline with no cloud dependency.
- Graph nodes and edges are deterministic, sorted, and schema-versioned.
- A changed entity can be traversed to related endpoints, services, tests, ADRs, ledger transactions, config keys, deploy surfaces, dependencies, observability signals, hotspots, and security boundaries.
- Empty or missing metadata overlays degrade gracefully.
- New migrations are idempotent and preserve existing ledger data.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo nextest run --lib --bins --workspace`, and `changeguard verify` pass.
- After source edits, `cargo install --path .` succeeds and the installed binary can run the new graph smoke command.

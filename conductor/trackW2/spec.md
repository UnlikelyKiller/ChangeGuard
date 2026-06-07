# Track W2 Spec: Public API Endpoint Ownership, Auth, and Consumer Graph

## Background

Public API and endpoint tracking currently scores 7/10. ChangeGuard has route extraction and contract matching, but endpoint ownership, auth requirements, request/response schema links, and consumer relationships are incomplete.

## Objective

Raise endpoint tracking to a reasonable 9/10 by linking route handlers, OpenAPI operations, schemas, auth requirements, owning services, and consumers into the entity graph.

## Proposed Design

1. Add an `api_endpoint_contract` graph projection that joins method, path, handler symbol, framework, source evidence, OpenAPI operation, request schema, response schema, service owner, and confidence.
2. Extract auth requirements from common Rust route/middleware patterns and allow config overlays for framework-specific or manually declared auth.
3. Detect consumers from HTTP callsites, generated clients, OpenAPI imports, sibling schema federation, and configured external consumers.
4. Add impact rules for method/path removal, response schema changes, auth weakening, consumer-visible breaking changes, and handler ownership changes.
5. Add `changeguard endpoints --json` and `changeguard endpoints --changed` for direct review.

## Critical Files

| File | Expected work |
|---|---|
| `src/index/routes.rs` | Normalize endpoint extraction into graph-ready records |
| `src/index/languages/rust/routes.rs` | Extend Rust route/auth evidence extraction |
| `src/contracts/parser.rs` | Link parsed schemas to endpoint graph nodes |
| `src/contracts/matcher.rs` | Feed endpoint contract changes into impact |
| `src/impact/enrichment/` | Add endpoint/auth/consumer risk rules |
| `src/commands/` and `src/cli.rs` | Add endpoint review commands |

## Definition of Done

- Endpoint records include method, path, handler, auth requirement, schemas, service, consumers, evidence, and confidence where available.
- Missing auth or consumer data is reported as unknown, not guessed as absent.
- Breaking endpoint changes produce explicit impact reasons and mapped verification recommendations.
- JSON output is stable enough for CI and AI-Brains ingestion.
- Target score after completion: 9/10.

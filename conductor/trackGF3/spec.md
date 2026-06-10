# Track GF3: Native Graph Loader Phase Extraction

## Objective

Break the monolithic `build_native_graph` flow in `src/index/graph_loader.rs` into explicit, testable phases while preserving graph output and indexing idempotence. The user-supplied analysis reports roughly 1353 lines with one 1300-line function spanning files, symbols, edges, routes, dependencies, deployments, environment variables, and related graph enrichment.

## Evidence

- User analysis ranks `src/index/graph_loader.rs` as refactor need 8/10 due to one huge sequential procedure and no direct unit-test structure.
- `changeguard hotspots trend` shows `src/index/graph_loader.rs` as the top current hotspot.
- `changeguard hotspots explain src/index/graph_loader.rs` reports temporal coupling above the 70% inspection threshold with `src/coverage/services.rs` at 0.76 and broad directory couplings.
- `.changeguard/reports/latest-impact.json` lists `src/index/graph_loader.rs` as top hotspot with complexity 61 and weighted frequency 13.59.
- Verified 2026-06-09: `build_native_graph` spans lines 24–1305 of the file. Two top-level helpers already live after it (`run_community_louvain`, `resource_matches_service`) — extraction should leave them as siblings of the new phase functions, not nest them.

## Scope

Required phase boundaries:

- `phase_files`: normalize repo paths, write project file nodes, prune stale file nodes.
- `phase_symbols`: write symbol nodes and symbol-to-file edges.
- `phase_call_edges`: write references, imports, calls, and ownership edges.
- `phase_routes`: write endpoint/route/auth/schema nodes and links.
- `phase_dependencies`: parse Cargo/npm/Python lock data and write package/dependency/advisory edges.
- `phase_deployments`: write deployment surfaces and service links.
- `phase_environment`: write config/env var keys and service ownership links.
- `phase_observability`: write metrics, SLOs, alerts, dashboards, and source-file links.
- `phase_security`: write policy, principal, action, resource, and protected-resource nodes, including orphan pruning.
- Shared context type: one internal `GraphLoadContext` carrying repo root, database handle, parsed inputs, and counters.

## Non-Goals

- Do not redesign the Cozo schema.
- Do not change node IDs, edge kinds, or serialized graph docs.
- Do not add network-backed dependency analysis.
- Do not mix this with `IndexOrchestrator` decomposition; that is GF6.

## Implementation Notes

- Extract by moving contiguous blocks into private functions first.
- Add phase counters so tests can assert idempotence without scraping all graph rows.
- Keep deterministic sorting before writes.
- Treat orphan pruning as high-risk and write tests before changing it.

## Verification Strategy

Targeted:

- `cargo test index::graph_loader`
- Integration tests for `index --analyze-graph`, `dependencies list`, `security boundaries`, `observability coverage`, and `ledger graph` if affected.
- `changeguard index --analyze-graph` on ChangeGuard itself.
- Read-side smokes beyond the listed surfaces — `endpoints --json`, `services diff`, `data-models impact --changed`, and a `viz --output` render — since all of them consume the graph this track rewrites.

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `build_native_graph` reads as an orchestration sequence of named phases.
- Each phase has at least one focused test, fixture, or integration smoke tied to its graph output.
- Re-running graph indexing is idempotent.
- Known graph surfaces still return non-empty data where fixtures exist.
- Final verification and reinstall pass.

## Risks

- Graph loader changes can silently orphan nodes or duplicate edges.
- Path normalization bugs can break changed-file matching for observability/security surfaces.
- Over-extraction can create too many tiny helpers without making phase ownership clearer.

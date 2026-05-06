# Track 48: Wire coverage.services Config + Fix Unused Parameters

## Overview
The `ServicesConfig` struct at `src/config/model.rs:570` defines `enabled` (bool, default false) and `cross_service_elevation_threshold` (u32, default 2), but these knobs are never consulted. Indexing and impact invoke service inference unconditionally regardless of the config. Additionally, the `index_services()` function calls `infer_services()` which accepts `call_graph` and `topology` parameters — `call_graph` is used for worker detection but the early-return at `services.rs:24` checks `call_graph.edges.is_empty()` alongside `routes.is_empty()`, meaning repos without routes AND without call graph edges get zero services even if topology has ServiceRoot classifications.

## Objectives
- Wire `coverage.services.enabled` into the indexing and impact pipelines so service inference is gated by config.
- Wire `cross_service_elevation_threshold` into `analyze_risk()` so it controls when "multi-service blast radius" risk is triggered.
- Fix `infer_services()` to not short-circuit when topology has explicit ServiceRoot directories but routes and call graph are empty.

## Success Criteria
- Setting `coverage.services.enabled = false` skips service inference entirely during both `index` and `impact`.
- Setting `cross_service_elevation_threshold = 5` only elevates risk when 5+ services are affected.
- Topology-only ServiceRoot directories create services even without routes or call graph edges.
- New tests: config wiring gating, threshold behavior, topology-only service creation.
- CI gate passes.

## Architecture
- `src/commands/index.rs` — Check `config.coverage.services.enabled` before calling `index_services()`.
- `src/commands/impact.rs` — Check `config.coverage.services.enabled`; the orchestrator's `ServiceProvider` enrichment already has a graceful-degradation path; add config gate.
- `src/impact/analysis.rs` — Replace hardcoded threshold (currently 2 at line 479) with `config.coverage.services.cross_service_elevation_threshold`.
- `src/coverage/services.rs` — Fix `infer_services()` early-return to account for topology-only ServiceRoot definitions.

## Testing Strategy
- **Red commit**: Tests asserting gating behavior and threshold sensitivity.
- **Green commit**: Wire config knobs. Verify all tests pass.

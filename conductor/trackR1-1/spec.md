# Track R1-1: Impact Orchestrator Extraction & Decomposition

## Overview
Decompose the monolithic `src/commands/impact.rs` (2,166 lines) into a modular architecture. Extract the core coordination logic into an `ImpactOrchestrator` and move enrichment logic into specialized providers.

## Objectives
- Extract orchestration logic to `src/impact/orchestrator.rs`.
- Decompose `populate_*` functions into `src/impact/enrichment/`.
- Abstract recurring SQL patterns into `src/state/storage.rs`.
- Rehome extracted logic to appropriate core modules (`src/index/analysis.rs`, `src/impact/analysis.rs`).

## Success Criteria
- `src/commands/impact.rs` reduced to < 300 lines (CLI wrapper only).
- `ImpactOrchestrator` successfully coordinates the full enrichment pipeline.
- All existing tests (unit and integration) pass.
- New unit tests for individual enrichment providers.

## Architecture
- `src/commands/impact.rs`: CLI argument parsing and `HumanOutput` invocation.
- `src/impact/orchestrator.rs`: COORDINATOR. Manages DB connections, file maps, and provider execution.
- `src/impact/enrichment/`:
    - `mod.rs`: Trait definition for `EnrichmentProvider`.
    - `api.rs`: API route discovery logic.
    - `observability.rs`: Log/telemetry/error-handling deltas.
    - `coupling.rs`: Structural/Temporal/DataFlow coupling.
    - `infra.rs`: Infra/EnvVar dependencies.
    - `services.rs`: Service map derivation.

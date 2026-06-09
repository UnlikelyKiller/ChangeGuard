# Track RE2: Modularize `src/impact/analysis/mod.rs`

## Objective
Decompose the project's largest file (2,281 lines) into a set of specialized `ImpactProvider` implementations.

## Requirements
- **Registry Pattern**: Update `ImpactOrchestrator` to use a dynamic registry of analysis providers.
- **Provider Extraction**: Move logic for Git, Dependencies, Semantic, and Temporal analysis into dedicated files in `src/impact/analysis/`.
- **Packet Logic**: Ensure the `ImpactPacket` generation remains consistent across providers.

## Definition of Done (DoD)
- [ ] `src/impact/analysis/mod.rs` is reduced to < 400 lines (orchestration only).
- [ ] Logic for each analysis type is in its own file.
- [ ] `changeguard scan --impact` returns identical results for existing test scenarios.
- [ ] Integration tests in `tests/risk_analysis.rs` pass.

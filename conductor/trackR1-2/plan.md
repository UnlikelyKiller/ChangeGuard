## Plan: Track R1-2 Monolithic Analysis Decomposition

### Phase 1: Infrastructure & Foundation
- [x] **Task 1.1**: Define `RiskProvider` trait and `RiskImpact` struct in `src/impact/providers/mod.rs`.
- [x] **Task 1.2**: Update `ImpactPacket` with a helper method to apply `RiskImpact` (encapsulating reason push and weight addition).
- [x] **Task 1.3**: Implement the `ImpactOrchestrator` registry and basic delegation logic.

### Phase 2: Extraction (Low Complexity)
- [x] **Task 2.1**: Extract "Protected Paths" logic into `PathProvider`.
- [x] **Task 2.2**: Extract "File/Symbol Volume" logic into `VolumeProvider`.
- [x] **Task 2.3**: Verify Phase 2 with unit tests and ensure `analyze_risk` calls these providers.

### Phase 3: Extraction (Middle Complexity)
- [x] **Task 3.1**: Extract "CI Pipeline" and "CI Self-Awareness" into `CiProvider`.
- [x] **Task 3.2**: Extract "Environment" and "Observability" logic into respective providers.
- [x] **Task 3.3**: Verify Phase 3 with unit tests.

### Phase 4: Extraction (High Complexity)
- [x] **Task 4.1**: Extract "API Surface" (entrypoints/handlers) into `ApiSurfaceProvider`.
- [x] **Task 4.2**: Extract "Coupling" (structural/data-flow) into `CouplingProvider`.
- [x] **Task 4.3**: Extract "Infrastructure" logic.

### Phase 5: Finalization & Cleanup
- [x] **Task 5.1**: Move all relevant tests from `src/impact/analysis.rs` to their provider modules. (Note: Kept in analysis.rs as integration tests to verify the full registry, added unit tests to providers).
- [x] **Task 5.2**: Shrink `analyze_risk` to a pure orchestrator entry point.
- [x] **Task 5.3**: Run full regression suite and ensure green CI gate.

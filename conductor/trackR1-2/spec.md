# Specification: Track R1-2 Monolithic Analysis Decomposition

## Goal
Decompose the monolithic `src/impact/analysis.rs` (~3,000 lines) into a modular, testable architecture using the **Orchestrator-Provider** pattern. This improves maintainability, reduces cognitive load for developers, and allows for isolated testing of risk scoring logic.

## Requirements
- Define a `RiskProvider` trait to standardize risk analysis components.
- Implement a `RiskImpact` struct to encapsulate weight and reasons.
- Move specific risk categories (e.g., Protected Paths, Volume, CI) into discrete provider implementations.
- Refactor the central `analyze_risk` function to act as an orchestrator that delegates to these providers.
- Maintain behavioral parity: the risk scores and reasons produced must remain identical to the current implementation.
- Zero `unwrap()` or `expect()` in production code.

## Architecture
- **Infrastructure**: `src/impact/providers/mod.rs` (Trait and registry).
- **Implementations**:
  - `src/impact/providers/path_provider.rs` (Protected Paths)
  - `src/impact/providers/volume_provider.rs` (File/Symbol Volume)
  - `src/impact/providers/ci_provider.rs` (CI/CD Config)
  - ... (and others as defined in the refactor plan)

## Testing Strategy
- **Unit Tests**: Each provider will have dedicated unit tests verifying its specific risk logic.
- **Regression Tests**: Use existing tests from `analysis.rs` (moved or mirrored) to ensure the total risk score for complex diffs remains unchanged.
- **Orchestrator Tests**: Verify that the orchestrator correctly collects and applies impacts from all registered providers.

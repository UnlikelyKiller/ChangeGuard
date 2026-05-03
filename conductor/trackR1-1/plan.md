# Implementation Plan - Track R1-1: Impact Orchestrator Extraction

## Goal
Decompose the monolithic `src/commands/impact.rs` into a clean, modular architecture centered around an `ImpactOrchestrator` and a suite of `EnrichmentProviders`.

## Proposed Changes

### 1. Storage Layer Abstractions [src/state/storage.rs](file:///c:/dev/ChangeGuard/src/state/storage.rs)
- Add `get_active_file_id_map(&self) -> Result<HashMap<PathBuf, i64>>` to centralize path-to-ID mapping.
- Add `table_exists(&self, name: &str) -> Result<bool>` to simplify feature-presence checks.

### 2. Orchestrator Foundation [src/impact/orchestrator.rs] [NEW]
- Define `ImpactOrchestrator` struct.
- Implement `run(&self, changes: Vec<FileChange>, config: &Config) -> Result<ImpactPacket>`.
- Handle DB connection lifecycle and packet initialization.

### 3. Enrichment Provider Pattern [src/impact/enrichment/mod.rs] [NEW]
- Define `EnrichmentProvider` trait.
- Define `EnrichmentContext` containing DB handle, file map, project metadata, and a thread-safe warning collector.

### 4. Resilient Execution Engine [src/impact/orchestrator.rs]
- Implement a `ProviderRegistry` to manage the list of active enrichment steps.
- **Error Isolation**: Wrap each provider execution in a `Result` handler. If a provider fails, log a warning to the `ImpactPacket.analysis_warnings` and continue with other enrichments (Graceful Degradation).
- **Hardened Data Access**: Orchestrator must validate that all paths in the `FileChange` set are normalized and within the project root before passing to providers.

### 5. Modularization (Migration of logic from `impact.rs`)
Extract the following logic into `src/impact/enrichment/`:
- **API**: `populate_api_contracts` -> `api.rs`
- **Observability**: `populate_observability_deltas` -> `observability.rs`
- **Coupling**: `populate_couplings` -> `coupling.rs`
- **Infrastructure**: `populate_infra` -> `infra.rs`
- **Services**: `populate_service_map` -> `services.rs`

### 6. Logic Rehoming
- Move `analyze_changed_file` logic to `src/index/analysis.rs`.
- Move risk scoring coordination to `src/impact/analysis.rs`.

### 7. CLI Wrapper [src/commands/impact.rs](file:///c:/dev/ChangeGuard/src/commands/impact.rs)
- Refactor `execute_impact` to:
  1. Load config and rules.
  2. Instantiate `ImpactOrchestrator`.
  3. Call `orchestrator.run()`.
  4. Call `human::print_impact_summary()`.

## Verification Plan

### Automated Tests
- `cargo test`: Ensure all existing unit and integration tests pass.
- New unit tests for each provider in `src/impact/enrichment/`.
- `tests/risk_analysis.rs`: Integration test for the full orchestrator flow.

### Manual Verification
- Run `changeguard impact` on a known dirty repo (e.g., ChangeGuard itself) and compare output with current baseline.

## Definition of Done (DoD)

- [ ] **SRP Compliance**: `src/commands/impact.rs` is < 300 lines and contains zero business logic.
- [ ] **Modularity**: New enrichment providers are located in `src/impact/enrichment/` and implement the `EnrichmentProvider` trait.
- [ ] **Resilience**: A failure in one enrichment provider (e.g., `api.rs`) does not prevent other providers (e.g., `infra.rs`) from completing.
- [ ] **Testability**: Every enrichment provider has a dedicated unit test suite with mocked storage states.
- [ ] **Zero Regression**: `tests/risk_analysis.rs` passes with identical risk scoring results compared to the monolithic version.
- [ ] **Architecture Docs**: `src/impact/mod.rs` contains a high-level overview of the orchestrator/provider lifecycle.
- [ ] **Clean CI**: `cargo clippy` and `cargo fmt` pass with zero warnings.

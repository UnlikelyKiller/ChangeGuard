# Implementation Plan - Track 48: Wire coverage.services Config

## Goal
Connect the existing but unused `ServicesConfig` fields (`enabled`, `cross_service_elevation_threshold`) to the service inference and risk analysis pipelines.

## Proposed Changes

### 1. Config Gate in Indexing [src/commands/index.rs]
- Find where `index_services()` is called (around line 112):
  ```rust
  // Before:
  let svc_stats = index.index_services()?;
  // After:
  let svc_stats = if config.coverage.services.enabled {
      index.index_services()?
  } else {
      info!("Service inference disabled by config.");
      ServiceIndexStats { services_inferred: 0, files_assigned: 0 }
  };
  ```

### 2. Config Gate in Impact [src/commands/impact.rs]
- The `ServiceProvider` enrichment already checks `table_exists_and_has_data("project_files")`. Add an additional check:
  - Either pass config through `EnrichmentContext` to `ServiceProvider::enrich()`, or
  - Check `config.coverage.services.enabled` before the orchestrator runs, and disable the provider.
- Simplest approach: In `execute_impact()`, after loading config but before running the orchestrator, check `config.coverage.services.enabled`. If false, skip service enrichment by not including the `ServiceProvider` in the orchestrator. The `ImpactOrchestrator::with_builtins()` includes it; add a config-gated variant.

### 3. Threshold Wiring [src/impact/analysis.rs]
- At line 479: `if count >= 2 {` — replace 2 with `config.coverage.services.cross_service_elevation_threshold as usize`:
  ```rust
  let threshold = config.coverage.services.cross_service_elevation_threshold as usize;
  if count >= threshold {
  ```

### 4. Topology-Only Service Creation [src/coverage/services.rs]
- At line 24, change the early-return condition:
  ```rust
  // Before:
  if routes.is_empty() && call_graph.edges.is_empty() {
      return Vec::new();
  }
  // After:
  if routes.is_empty() && call_graph.edges.is_empty() && topology.classifications.is_empty() {
      return Vec::new();
  }
  ```
  This allows ServiceRoot-classified directories to create service entries even without routes.

### 5. Tests
- `test_config_services_disabled_skips_inference`: Set enabled=false, run index, assert no services created.
- `test_cross_service_elevation_threshold_respected`: Set threshold=5, assert 4 affected services don't trigger elevation.
- `test_topology_service_root_without_routes`: Topology with ServiceRoot but no routes → service created.
- `test_config_defaults_preserved`: Default config produces identical behavior to current.

## Verification Plan

### Automated Tests
- `cargo test` in affected modules.
- `cargo test --workspace`

## Definition of Done (DoD)
- [x] **Config Gate**: `coverage.services.enabled = false` skips service inference in both index and impact.
- [x] **Threshold Wired**: `cross_service_elevation_threshold` controls risk elevation trigger.
- [x] **Topology Fix**: ServiceRoot directories create services without routes.
- [x] **Test Coverage**: 4+ new tests.
- [x] **Zero Regression**: All existing tests pass.
- [x] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

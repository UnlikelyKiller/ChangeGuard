# ChangeGuard Refactoring Plan

## Overview
This document outlines the strategy for decomposing "god files" in the ChangeGuard project to improve maintainability, testability, and architectural clarity.

## Current "God Files" (Ranked by Size)

| File Path | Line Count | Size (KB) | Risk/Refactor Priority |
| :--- | :--- | :--- | :--- |
| **`src\impact\analysis.rs`** | **2,949** | 113.1 | **Critical**: Centralized risk scoring logic. |
| **`src\state\migrations.rs`** | **2,093** | 97.8 | **Medium**: Growing list of historical schema changes. |
| **`src\index\project_index.rs`** | **1,749** | 74.6 | **High**: Core indexing logic. |
| **`src\impact\packet.rs`** | **1,497** | 54.5 | **Medium**: Mixed data structure and serialization logic. |
| **`src\index\languages\rust.rs`** | **1,463** | 60.7 | **High**: Language-specific extraction complexity. |

---

## Deep Dive: `src\impact\analysis.rs` Refactoring

### Current State
`src\impact\analysis.rs` contains a monolithic `analyze_risk` function (~700 lines) that handles dozens of distinct risk categories, from "Protected Paths" to "CI Self-Awareness." It is hard to test in isolation and violates the Single Responsibility Principle (SRP).

### Proposed Architecture: The Orchestrator-Provider Pattern

We will transition from a procedural switch-like function to a registry of **Risk Providers**.

#### 1. Define the `RiskProvider` Trait
```rust
pub trait RiskProvider: Send + Sync {
    fn name(&self) -> &str;
    fn analyze(&self, packet: &ImpactPacket, config: &Config, rules: &Rules) -> Result<RiskImpact>;
}

pub struct RiskImpact {
    pub weight: u32,
    pub reasons: Vec<String>,
}
```

#### 2. Modular Decomposition
Create a new directory `src/impact/providers/` and move logic into discrete implementations:
- `path_provider.rs`: Protected path logic.
- `volume_provider.rs`: File and symbol volume scoring.
- `api_surface_provider.rs`: Entrypoints, handlers, and public API changes.
- `coupling_provider.rs`: Structural and data-flow coupling.
- `observability_provider.rs`: Logging, error handling, and telemetry delta analysis.
- `ci_provider.rs`: CI/CD configuration and self-awareness.
- `infra_provider.rs`: Deployment manifests and infrastructure drift.

#### 3. The `ImpactOrchestrator`
Replace the logic in `analysis.rs` with a lean orchestrator:
```rust
pub struct ImpactOrchestrator {
    providers: Vec<Box<dyn RiskProvider>>,
}

impl ImpactOrchestrator {
    pub fn analyze(&self, packet: &mut ImpactPacket, ...) -> Result<()> {
        for provider in &self.providers {
            let impact = provider.analyze(packet, ...)?;
            packet.apply_impact(impact);
        }
        // Final scoring logic
        packet.finalize_risk_level();
        Ok(())
    }
}
```

### Refactoring Steps
1. **Infrastructure**: Create `src/impact/providers/mod.rs` and the `RiskProvider` trait.
2. **Phase 1 (Low Risk)**: Extract "Protected Paths" and "Volume" into providers.
3. **Phase 2 (Complexity)**: Extract "API Surface" and "Coupling" (requires careful handling of state/caps).
4. **Phase 5 (Enrichment)**: Extract "CI" and "Observability" logic.
5. **Phase 4 (Finalize)**: Refactor `analyze_risk` to use the registry and move existing tests to their respective provider modules.

---

## Other Refactoring Targets

### `src\state\migrations.rs`
- **Action**: Split into a `src/state/migrations/` directory.
- **Strategy**: Use one file per migration (e.g., `v001_initial.rs`, `v002_add_ledger.rs`). Use a macro or registry to collect them.

### `src\index\project_index.rs`
- **Action**: Decompose by indexing phase.
- **Strategy**: Separate "Git Interaction," "AST Parsing Orchestration," and "Graph Insertion" into sub-modules.

### `src\index\languages\*.rs`
- **Action**: Extract common patterns.
- **Strategy**: Create an `ExtractionSuite` trait in `src/index/languages/mod.rs` to standardize how symbols, imports, and docstrings are extracted across languages.

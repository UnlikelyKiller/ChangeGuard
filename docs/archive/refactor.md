# Refactoring Proposal: `src\commands\impact.rs`

## Current State Analysis

The file `src\commands\impact.rs` currently serves as a monolithic implementation for the entire impact analysis lifecycle. At **2,166 lines**, it violates several core engineering principles established for the ChangeGuard project.

### Critical Issues

1.  **Violation of SRP (Single Responsibility Principle)**:
    *   The `execute_impact` function handles CLI orchestration, config loading, analysis coordination, risk scoring, and persistence.
    *   It contains 15+ `populate_*` functions that mix business logic with low-level SQL query construction.
2.  **Significant Code Duplication (WET)**:
    *   Recurring boilerplate for database connection handling, table existence checks, and file-to-ID mapping across almost all enrichment functions.
3.  **High Cognitive Load**:
    *   The massive file size makes it difficult to reason about the analysis pipeline or find specific logic.
4.  **Poor Testability**:
    *   Analysis logic is trapped in private functions within a command module, making unit testing difficult without triggering the full CLI path.

## Proposed Changes

### 1. Extract Orchestration
Move the core orchestration logic from `src\commands\impact.rs` to a new `ImpactOrchestrator` in `src\impact\orchestrator.rs`. The command module should be a thin wrapper that:
- Parses CLI flags.
- Calls the orchestrator.
- Formats the output for the human user.

### 2. Module Decomposition (Enrichment Providers)
Move the `populate_*` functions into specialized modules under `src\impact\enrichment\`:
- **`api.rs`**: API route discovery.
- **`observability.rs`**: Logging, telemetry, and error-handling deltas.
- **`coupling.rs`**: Structural, temporal, and data-flow coupling.
- **`infra.rs`**: Infrastructure and environment variable dependencies.
- **`services.rs`**: Cross-service impact mapping.

### 3. Abstract Database Patterns
Create a centralized utility to handle recurring SQL patterns:
- `storage.get_active_file_id_map()`
- `storage.table_exists_and_has_data(table_name)`

### 4. Logic Rehoming
- Move `analyze_changed_file` to `src\index\analysis.rs` (SRP: single-file symbol extraction belongs in `index`).
- Consolidate risk scoring logic into `src\impact\analysis.rs`.

## Expected Benefits

- **Maintainability**: Smaller, focused files are easier to understand and audit.
- **Extensibility**: Adding new analysis types (e.g., Ledger-based facts) becomes a modular task rather than modifying a monolithic function.
- **Testability**: Enrichment steps can be unit-tested in isolation using mock database states.
- **Alignment**: Brings the codebase into compliance with the project's **Engineering Principles** and **SRP Mandates**.

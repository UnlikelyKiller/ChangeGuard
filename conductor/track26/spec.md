# Track 26: Predictive Verification (Dependency-Aware)

## Goal
Use temporal coupling (from Track 23) and structural imports (from Phase 1) to predict which files *should* be verified even if they haven't changed in the current commit. Expand the verification plan based on this combined impact graph.

## Context
ChangeGuard currently verifies files based purely on explicit file modifications. However, modern systems fail due to non-obvious dependencies. If `src/lib.rs` changes, structurally dependent code (like `src/cli.rs`) and temporally coupled code (like `tests/integration.rs`) might break. This track bridges the gap by building a deterministic, depth-bounded prediction engine.

## Requirements

### 1. Prediction Engine (`src/verify/predict.rs`)
- **Core Logic**: Define a predictive engine that outputs a list of `PredictedFile` records.
- **Inputs**:
  - `ImpactPacket` (contains structural imports and direct file changes).
  - `TemporalResult` or temporal coupling map (computed from `src/impact/temporal.rs`).
- **Algorithm**: A file is predicted to be impacted if:
  - It is known to import one of the actually changed files (Structural Impact).
  - It is frequently changed alongside one of the actually changed files according to temporal history (Temporal Impact).
- **Depth Constraints (KISS/YAGNI)**: Restrict the impact propagation to **Depth 1** initially. Do not build an unbounded recursive graph traversal. A simple custom adjacency list (e.g., `HashMap<PathBuf, Vec<PathBuf>>`) is sufficient. Do not pull in heavy graph crates like `petgraph` yet.
- **Determinism**: The prediction algorithm must be 100% deterministic. Resolve ties and output order by sorting file paths alphabetically.

### 2. Verification Plan Integration (`src/verify/plan.rs`)
- **Plan Expansion**: Extend the `VerificationPlan` generation to ingest predicted files.
- **Rule Evaluation**: Apply path rules (`PathRule` in `rules.toml`) against the *predicted* files as if they had changed.
- **Traceability**: If a verification command is added due to a prediction, the `VerificationStep::description` MUST explicitly state this (e.g., `"Predicted impact on tests/auth.rs (Temporal): cargo test"`). This ensures no "magic" or invisible rule evaluations occur.

### 3. CLI Integration (`src/commands/verify.rs`)
- **Execution Flow**: Wire the temporal analysis and structural impact data into the `verify` command prior to plan construction.
- **Graceful Degradation**: If temporal analysis is unavailable (e.g., shallow git clone, fewer than 10 commits), the prediction engine MUST emit a deterministic warning (e.g., `degraded: true, reason: "Insufficient history"`) and fall back to structural-only prediction. It MUST NOT silently fail or crash.
- **Opt-Out Flag**: Provide a `--no-predict` flag in the CLI to force strict file-based verification, preserving an escape hatch for the user.

### 4. Adherence to Engineering Principles
- **SRP**: Prediction logic stays in `predict.rs`. Plan construction stays in `plan.rs`.
- **Idiomatic Rust**: Return `miette::Result` for any fallible steps. Absolutely no `unwrap()` or `expect()` in production paths.
- **Error Visibility**: Errors must clearly describe *what* failed (e.g., "Failed to load temporal history") and provide remediation steps (e.g., "Run git fetch --unshallow").

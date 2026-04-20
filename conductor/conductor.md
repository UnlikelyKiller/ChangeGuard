# ChangeGuard Conductor

## Active Tracks

*   **Track 19: Reset and Recovery Completion** (Next)
    *   Status: Planning
    *   Spec: `conductor/track19/spec.md`
    *   Plan: `conductor/track19/plan.md`
    *   Audit2 findings: Functional finding 1, Phase 14 gap, Source-tree `reset` deficiency
    *   Key additions: real `reset` command, `src/commands/reset.rs`, derived-state cleanup, optional config/rules removal, recovery path for broken local state

*   **Track 20: Determinism and Error Visibility Hardening** (After Track 19)
    *   Status: Planning
    *   Spec: `conductor/track20/spec.md`
    *   Plan: `conductor/track20/plan.md`
    *   Audit2 findings: Functional findings 3 and 4, Determinism gaps, Error Visibility gaps
    *   Key additions: validated rule loading, no silent config/rules fallback, explicit partial-analysis warnings in impact packets, deterministic warning ordering

*   **Track 21: Verification Process Hardening** (After Track 20)
    *   Status: Planning
    *   Spec: `conductor/track21/spec.md`
    *   Plan: `conductor/track21/plan.md`
    *   Audit2 findings: Functional finding 2, Phase 12 and Phase 15 gaps
    *   Key additions: `verify/runner.rs`, `verify/timeouts.rs`, process-policy enforcement, reduced shell dependence, dedicated platform verification tests

*   **Track 22: Structural Completion and Plan Reconciliation** (Last of Phase 1)
    *   Status: Planning
    *   Spec: `conductor/track22/spec.md`
    *   Plan: `conductor/track22/plan.md`
    *   Audit2 findings: Functional findings 5, 6, 7, remaining source/doc/test layout gaps
    *   Key additions: scan diff-summary integration, symbol persistence/storage seams, remaining planned modules or documented shims, missing docs artifacts, black-box CLI coverage, `cargo deny`

## Milestone E: Historical Intelligence Tracks (Completed)

## Milestone F: Predictive Verification Tracks

*   **Track 26: Predictive Verification (Dependency-Aware)** (Next)
    *   Status: Planning
    *   Spec: `conductor/track26/spec.md`
    *   Plan: `conductor/track26/plan.md`
    *   Goal: Use temporal coupling and structural imports to predict which files *should* be verified even if they haven't changed.
    *   Key additions: `src/verify/predict.rs`, Graph-based impact propagation, verification plan expansion logic.

## Execution Order

1. Track 19 (Reset and Recovery Completion)
2. Track 20 (Determinism and Error Visibility Hardening)
3. Track 21 (Verification Process Hardening)
4. Track 22 (Structural Completion and Plan Reconciliation)
5. Track 26 (Predictive Verification)

## Completed Tracks

*   **Track 25: Hotspot Identification (Risk Density)**
    *   Status: Completed
    *   Spec: `conductor/track25/spec.md`
    *   Plan: `conductor/track25/plan.md`
    *   Goal: Combine change frequency (Track 23) with structural complexity (Track 24) to output Risk Maps.
    *   Key additions: `src/commands/hotspots.rs`, `changeguard hotspots` CLI command, Risk Density scoring engine, human-readable hotspot tables.

*   **Track 24: Complexity Indexing (Spike & Implementation)**
    *   Status: Completed
    *   Spec: `conductor/track24/spec.md`
    *   Plan: `conductor/track24/plan.md`
    *   Goal: Measure cognitive and cyclomatic complexity for functions and structs to weight impact risks.
    *   Key additions: `src/index/metrics.rs`, `NativeComplexityScorer` using tree-sitter, SQLite persistence for symbol complexity.

*   **Track 23: Temporal Intelligence (History Extraction)**
    *   Status: Completed
    *   Spec: `conductor/track23/spec.md`
    *   Plan: `conductor/track23/plan.md`
    *   Goal: Identify "Logical Coupling" between files by crawling git history.
    *   Key additions: `src/impact/temporal.rs`, `gix` 0.81.0 integration, deterministic affinity mapping, configurable commit depth.

*   **Track 13: Final Integration**
    *   Status: Completed with follow-up required
    *   Note: reset was treated as complete in the original track sequence, but the current repo still lacks a real `src/commands/reset.rs` implementation. This is now addressed by active **Track 19: Reset and Recovery Completion**.

*   **Track 12: UI/UX Refinement**
    *   Status: Completed

*   **Track 11: Ask Gemini Baseline**
    *   Status: Completed

*   **Track 10: State SQLite Persistence**
    *   Status: Completed

*   **Track 9: Change Risk Analysis Engine**
    *   Status: Completed

*   **Track 8: Determinism Contract and Subprocess Control**
    *   Status: Completed

*   **Track 7: Language-Aware Symbol Extraction**
    *   Status: Completed

*   **Track 6: Watch Mode and Batch Debouncing**
    *   Status: Completed

*   **Track 5: Basic Impact Packet Shell**
    *   Status: Completed

*   **Track 4: Git Scan Foundation**
    *   Status: Completed

*   **Track 3: Config and Rule Loading**
    *   Status: Completed

*   **Track 2: Doctor and Platform Detection**
    *   Status: Completed

*   **Track 1: Repo-Local State Layout and Init**
    *   Status: Completed

*   **Track 0: Bootstrap CLI Skeleton**
    *   Status: Completed

## Workflow

1.  **Plan**: `@architecture-planner` creates `conductor/trackN/plan.md`.
2.  **Push Plan**: Commit and push plan to `main`.
3.  **Implement**: `@generalist` (Implementer) creates a new branch and works on the task.
4.  **Review**: `@rust-triage-specialist` or `@frontend-reviewer` (Reviewer) audits the branch.
5.  **Iteration**: If review fails, Implementer fixes.
6.  **Merge**: If review passes, create PR or merge into `main`.
7.  **Next**: Start next track.

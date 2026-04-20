# ChangeGuard Conductor

## Milestone I: Phase 2 Remediation (Active)

*   **Track 30: Foundation & Safety Remediation**
    *   Status: Completed
    *   Spec: `conductor/track30/spec.md`
    *   Plan: `conductor/track30/plan.md`
    *   Goal: Restore green CI gates (fmt, clippy, tests), fix critical secret redaction/persistence issues, and remove production unwraps.
    *   Key additions: Green CI suite, secured redact-before-save persistence, zero production unwraps in impact/hotspots.

*   **Track 31: Intelligence & Determinism Hardening**
    *   Status: Completed
    *   Spec: `conductor/track31/spec.md`
    *   Plan: `conductor/track31/plan.md`
    *   Goal: Fix temporal first-parent traversal, add complexity degradation, and ensure deterministic sorting for hotspots.
    *   Key additions: First-parent traversal by default, complexity AST degradation, multiplication-based hotspot scoring, hotspot JSON/filtering.

*   **Track 32: Predictive Verification Completion**
    *   Status: Completed
    *   Spec: `conductor/track32/spec.md`
    *   Plan: `conductor/track32/plan.md`
    *   Goal: Implement structural prediction (removing placeholders) and add missing predictor tests and degradation warnings.
    *   Key additions: Structural import-based prediction, placeholder cleanup, comprehensive predictor tests, merged plan descriptions.

*   **Track 33: Federated Intelligence Completion** (High) (Next)
    *   Status: Planning
    *   Spec: `conductor/track33/spec.md`
    *   Plan: `conductor/track33/plan.md`
    *   Goal: Implement real cross-repo impact resolution with dependency edges, schema validation, and path confinement.
    *   Audit3 findings: Generic placeholder warnings, missing schema validation, lack of path confinement.

*   **Track 34: Narrative Reporting Completion** (High)
    *   Status: Planning
    *   Spec: `conductor/track34/spec.md`
    *   Plan: `conductor/track34/plan.md`
    *   Goal: Wire token budgeting and truncation annotations into Gemini execution, and add golden prompt tests.
    *   Audit3 findings: Token budget unused, truncation annotation absent, Gemini fallback incomplete.

*   **Track 35: LSP Daemon Resolution** (Critical)
    *   Status: Planning
    *   Spec: `conductor/track35/spec.md`
    *   Plan: `conductor/track35/plan.md`
    *   Goal: Replace the stub implementation with a fully-featured LSP server in `src/daemon/` (with Tokio runtime and lifecycle management).
    *   Audit3 findings: Daemon is just a watch wrapper, missing LSP features and lifecycle management.

## Active Tracks (Phase 1)

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

## Milestone F: Predictive Verification Tracks (Completed)

## Milestone G: IDE Integration Tracks (Completed)

## Milestone H: Cross-Repo and Narrative Reporting (Completed)

## Completed Tracks (Phase 2 - Partial/Needs Remediation)

*   **Track 29: Advanced Narrative Reporting (Gemini)** (Status: Partial)
*   **Track 28: Federated Intelligence (Cross-Repo)** (Status: Partial)
*   **Track 27: LSP-Lite ChangeGuard Daemon** (Status: Fail)
*   **Track 26: Predictive Verification (Dependency-Aware)** (Status: Fail/Partial)
*   **Track 25: Hotspot Identification (Risk Density)** (Status: Partial)
*   **Track 24: Complexity Indexing (Spike & Implementation)** (Status: Partial Pass)
*   **Track 23: Temporal Intelligence (History Extraction)** (Status: Partial Pass)

*   **Track 13: Final Integration** (Status: Completed with follow-up required)
*   **Track 12: UI/UX Refinement** (Status: Completed)
*   **Track 11: Ask Gemini Baseline** (Status: Completed)
*   **Track 10: State SQLite Persistence** (Status: Completed)
*   **Track 9: Change Risk Analysis Engine** (Status: Completed)
*   **Track 8: Determinism Contract and Subprocess Control** (Status: Completed)
*   **Track 7: Language-Aware Symbol Extraction** (Status: Completed)
*   **Track 6: Watch Mode and Batch Debouncing** (Status: Completed)
*   **Track 5: Basic Impact Packet Shell** (Status: Completed)
*   **Track 4: Git Scan Foundation** (Status: Completed)
*   **Track 3: Config and Rule Loading** (Status: Completed)
*   **Track 2: Doctor and Platform Detection** (Status: Completed)
*   **Track 1: Repo-Local State Layout and Init** (Status: Completed)
*   **Track 0: Bootstrap CLI Skeleton** (Status: Completed)

## Workflow

1.  **Plan**: `@architecture-planner` creates `conductor/trackN/plan.md`.
2.  **Push Plan**: Commit and push plan to `main`.
3.  **Implement**: `@generalist` (Implementer) creates a new branch and works on the task.
4.  **Review**: `@rust-triage-specialist` or `@frontend-reviewer` (Reviewer) audits the branch.
5.  **Iteration**: If review fails, Implementer fixes.
6.  **Merge**: If review passes, create PR or merge into `main`.
7.  **Next**: Start next track.

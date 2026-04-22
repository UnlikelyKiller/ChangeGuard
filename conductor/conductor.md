# ChangeGuard Conductor

## Milestone L: Ledger Incorporation (Active)

*   **Track L1-1: Ledger Data Model & Migrations**
    *   Status: Completed
    *   Spec: `conductor/trackL1-1/spec.md`
    *   Plan: `conductor/trackL1-1/plan.md`
    *   Goal: Implement Phase L1 (Transaction Lifecycle & Data Model) - Data Model, Error types, and Migrations M11 & M12.
    *   Key additions: `src/ledger/types.rs`, `src/ledger/error.rs`, SQLite migrations M11-M12, `LedgerConfig`.

*   **Track L1-2: Transaction Lifecycle Management**
    *   Status: Completed
    *   Spec: `conductor/trackL1-2/spec.md`
    *   Plan: `conductor/trackL1-2/plan.md`
    *   Goal: Implement the core transaction lifecycle (start, commit, rollback, atomic) and CLI commands.
    *   Key additions: `src/ledger/db.rs`, `src/ledger/transaction.rs`, `src/ledger/session.rs`, `ledger start/commit/rollback/atomic`.

*   **Track L1-R: Ledger Phase 1 Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackL1-R/spec.md`
    *   Plan: `conductor/trackL1-R/plan.md`
    *   Goal: Address the high and medium severity findings from the Codex review for Phase L1.
    *   Key additions: Transactional safety, correct lifecycle states, robust path normalization, WAL concurrency, verification persistence, and CLI gaps.

*   **Track L2-1: Ledger Drift Detection**
    *   Status: Completed
    *   Spec: `conductor/trackL2-1/spec.md`
    *   Plan: `conductor/trackL2-1/plan.md`
    *   Goal: Integrate file watcher with the ledger to detect and record untracked changes (drift).
    *   Key additions: Watcher transaction checks, UNAUDITED record creation, drift counting, enhanced status reporting.

*   **Track L2-2: Ledger Reconciliation & Adoption**
    *   Status: Active
    *   Spec: `conductor/trackL2-2/spec.md`
    *   Plan: `conductor/trackL2-2/plan.md`
    *   Goal: Implement reconciliation and adoption commands to manage detected drift.
    *   Key additions: `ledger reconcile`, `ledger adopt`, drift transition logic, reconciliation provenance.

## Milestone J: Phase 2 Final Remediation (Completed)

*   **Track 36: Critical Remediation & Green CI**
    *   Status: Completed
    *   Spec: `conductor/track36/spec.md`
    *   Plan: `conductor/track36/plan.md`
    *   Goal: Restore `cargo fmt` and `cargo clippy`, fix silent hotspot row error dropping, and surface prediction degradation in `verify`.
    *   Key additions: Green CI pipeline, visible SQLite hotspot errors, user-visible verification prediction warnings.

*   **Track 37: LSP Daemon Functional Completion**
    *   Status: Completed
    *   Spec: `conductor/track37/spec.md`
    *   Plan: `conductor/track37/plan.md`
    *   Goal: Implement real Hover, CodeLens, real-time diagnostics, broken-stdin self-termination, and lifecycle tests, removing all placeholders.
    *   Key additions: Fully functional LSP handlers, `data_stale` surface handling, stdin auto-termination, robust daemon lifecycle tests.

*   **Track 38: Complexity & Temporal Hardening**
    *   Status: Completed
    *   Spec: `conductor/track38/spec.md`
    *   Plan: `conductor/track38/plan.md`
    *   Goal: Complete missing edge tests for complexity, add real temporal git fixtures, add dedicated hotspot tests, and document the arborist spike.
    *   Key additions: Arborist ADR, TS/syntax/unsupported-language complexity tests, temporal git-fixture testing, hotspot scoring tests.

*   **Track 39: Dependency & Federation Deepening**
    *   Status: Completed
    *   Spec: `conductor/track39/spec.md`
    *   Plan: `conductor/track39/plan.md`
    *   Goal: Upgrade structural prediction and dependency discovery to use current repo data, automate federation discovery, and use shared redaction.
    *   Key additions: Current-repo structural analysis, automated federation discovery in `scan`/`impact`, shared redaction model usage, path confinement edge tests.

*   **Track 40: Narrative Refinement**
    *   Status: Completed
    *   Spec: `conductor/track40/spec.md`
    *   Plan: `conductor/track40/plan.md`
    *   Goal: Rework narrative prompt construction to avoid nesting and ensure fallback write failures are visible.
    *   Key additions: Flat, structured narrative prompts, robust error handling for fallback report persistence.

## Milestone I: Phase 2 Remediation (Completed)

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

*   **Track 33: Federated Intelligence Completion**
    *   Status: Completed
    *   Spec: `conductor/track33/spec.md`
    *   Plan: `conductor/track33/plan.md`
    *   Goal: Implement real cross-repo impact resolution with dependency edges, schema validation, and path confinement.
    *   Key additions: Path confinement security, schema validation, dependency insertion, cross-repo impact resolution tests.

*   **Track 34: Narrative Reporting Completion**
    *   Status: Completed
    *   Spec: `conductor/track34/spec.md`
    *   Plan: `conductor/track34/plan.md`
    *   Goal: Wire token budgeting and truncation annotations into Gemini execution, and add golden prompt tests.
    *   Key additions: Strict token budgeting (409,600 chars), truncation annotations, robust Gemini execution with fallback artifacts, byte-for-byte deterministic prompt tests.

*   **Track 35: LSP Daemon Resolution**
    *   Status: Completed
    *   Spec: `conductor/track35/spec.md`
    *   Plan: `conductor/track35/plan.md`
    *   Goal: Replace the stub implementation with a fully-featured LSP server using `tower-lsp-server` and `tokio`.
    *   Key additions: Real-time diagnostic reporting, robust lifecycle (PID management), read-only WAL SQLite access, and native async trait implementation.

## Completed Tracks


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

# ChangeGuard Conductor

## Active Tracks

*   **Track 14: Critical Safety Fixes** (Next)
    *   Status: Completed (feat/critical-safety-fixes)
    *   Spec: `conductor/track14/spec.md`
    *   Plan: `conductor/track14/plan.md`
    *   Audit items: 1, 2, 3, 11 (CRITICAL + HIGH)
    *   Key additions: secret redaction with entropy check, verification planning, Gemini subprocess timeout, fix ALL production unwrap/expect

*   **Track 17: Engineering Quality** (Completed)
    *   Status: Completed (feat/engineering-quality)
    *   Spec: `conductor/track17/spec.md`
    *   Plan: `conductor/track17/plan.md`
    *   Audit items: 15, 17, 18, 19, 20 (MEDIUM)
    *   Key additions: remove unused deps, cross-platform tests, docs casing fix, DB schema expansion, config validation
    *   **Must run before Track 16** (DB schema needed by verify results and batch persistence)

*   **Track 15: Gemini Modes, Output Module, Git Classification** (Next)
    *   Status: Planning
    *   Spec: `conductor/track15/spec.md`
    *   Plan: `conductor/track15/plan.md`
    *   Audit items: 4, 5, 7, 8 (HIGH)
    *   Key additions: Gemini modes (analyze/suggest/review-patch), output module refactor (human+diagnostics, YAGNI on table), git classify fix (added/deleted/renamed)

*   **Track 16: Relationship Extraction, Watch Hardening, Verify Results** (After Track 15)
    *   Status: Planning
    *   Spec: `conductor/track16/spec.md`
    *   Plan: `conductor/track16/plan.md`
    *   Audit items: 6, 12, 13, 14 (HIGH + MEDIUM)
    *   Key additions: import/export extraction, runtime usage detection, watch ctrl+c + config integration, verify results persistence
    *   **Depends on Track 17** (DB schema for persistence)

*   **Track 18: Documentation, CI, and Polish** (Last)
    *   Status: Planning
    *   Spec: `conductor/track18/spec.md`
    *   Plan: `conductor/track18/plan.md`
    *   Audit items: 9, 10, 16, 21–25 (HIGH + LOW)
    *   Key additions: README, CI with caching, test dedup, Clock trait (not normalize fn), process policy, docs, fixtures

## Execution Order

1. Track 14 (Critical Safety Fixes)
2. Track 17 (Engineering Quality — DB schema needed by Track 16)
3. Track 15 (Output Module — structural refactor before feature additions)
4. Track 16 (Features that depend on new schema and output module)
5. Track 18 (Documentation and Polish — always last)

## Completed Tracks

*   **Track 13: Final Integration and Reset Command**
    *   Status: Completed

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
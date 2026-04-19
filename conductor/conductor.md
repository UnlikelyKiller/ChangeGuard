# ChangeGuard Conductor

## Active Tracks

*   **Track 14: Critical Safety Fixes** (Next)
    *   Status: Planning
    *   Spec: `conductor/track14/spec.md`
    *   Plan: `conductor/track14/plan.md`
    *   Audit items: 1, 2, 3, 11 (CRITICAL + HIGH)

*   **Track 15: Gemini Modes, Output Module, Git Classification** (Queued)
    *   Status: Planning
    *   Spec: `conductor/track15/spec.md`
    *   Plan: `conductor/track15/plan.md`
    *   Audit items: 4, 5, 7, 8 (HIGH)

*   **Track 16: Relationship Extraction, Watch Hardening, Verify Results** (Queued)
    *   Status: Planning
    *   Spec: `conductor/track16/spec.md`
    *   Plan: `conductor/track16/plan.md`
    *   Audit items: 6, 12, 13, 14 (HIGH + MEDIUM)

*   **Track 17: Engineering Quality** (Queued)
    *   Status: Planning
    *   Spec: `conductor/track17/spec.md`
    *   Plan: `conductor/track17/plan.md`
    *   Audit items: 15, 17, 18, 19, 20 (MEDIUM)

*   **Track 18: Documentation, CI, and Polish** (Queued)
    *   Status: Planning
    *   Spec: `conductor/track18/spec.md`
    *   Plan: `conductor/track18/plan.md`
    *   Audit items: 9, 10, 16, 21–25 (HIGH + LOW)

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
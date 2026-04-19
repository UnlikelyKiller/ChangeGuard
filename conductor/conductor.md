# ChangeGuard Conductor

## Active Tracks

*   **Track 11: Ask Gemini Baseline** (Active)
    *   Status: Planning
    *   Plan: `conductor/track11/plan.md`
    *   Implementation Branch: `feat/ask-gemini`

## Completed Tracks

*   **Track 10: State SQLite Persistence**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 9: Change Risk Analysis Engine**

*   **Track 7: Language-Aware Symbol Extraction**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 6: Watch Mode and Batch Debouncing**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 5: Basic Impact Packet Shell**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 4: Git Scan Foundation**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 3: Config and Rule Loading**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 2: Doctor and Platform Detection**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 1: Repo-Local State Layout and Init**
    *   Status: Completed
    *   PR: Merged into `main`

*   **Track 0: Bootstrap CLI Skeleton**
    *   Status: Completed
    *   PR: N/A (Pushed directly to main)

## Workflow

1.  **Plan**: `@architecture-planner` creates `conductor/trackN/plan.md`.
2.  **Push Plan**: Commit and push plan to `main`.
3.  **Implement**: `@generalist` (Implementer) creates a new branch and works on the task.
4.  **Review**: `@rust-triage-specialist` or `@frontend-reviewer` (Reviewer) audits the branch.
5.  **Iteration**: If review fails, Implementer fixes.
6.  **Merge**: If review passes, create PR or merge into `main`.
7.  **Next**: Start next track.

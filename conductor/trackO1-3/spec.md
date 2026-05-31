# Track O1-3: Git Hook Integration & UX Logic

## Objective
Wire the TUI (Track O1-1) and LLM Pipeline (Track O1-2) into the actual git lifecycle. Ensure the hook intercepts commits, applies the adaptive bypass logic, and provides a frictionless developer experience.

## Requirements
*   **Git Hook Generation:** Update `changeguard init` to generate a `commit-msg` hook. (A `commit-msg` hook is required because the LLM needs the user's commit message to infer the "WHY").
*   **Hook Execution Flow:**
    1. Read `.git/COMMIT_EDITMSG`.
    2. Read staged `git diff`.
    3. Call `IntentDrafter` (from O1-2).
    4. If confidence >= 0.85, silently write to CozoDB ledger and exit 0.
    5. If confidence < 0.85, launch `ratatui` UI (from O1-1). Wait for user acceptance.
*   **Adaptive Bypass:** Track consecutive "trivial" skips. If a user manually skips with reason "trivial", auto-accept the next 2 trivial-looking commits without prompting.
*   **Config Controls:** Support `[intent]\nrequired = "always" | "never"` in `.changeguard/config.toml` to allow teams to override hook behavior.

## Definition of Done (DoD)
*   [ ] The `commit-msg` hook correctly intercepts the commit process.
*   [ ] High-confidence LLM drafts are silently committed to the ledger.
*   [ ] Low-confidence LLM drafts successfully launch the interactive TUI.
*   [ ] Adaptive bypass correctly suppresses the TUI for consecutive trivial commits.
*   [ ] Bypassing with `git commit --no-verify` or config `required = "never"` works seamlessly.
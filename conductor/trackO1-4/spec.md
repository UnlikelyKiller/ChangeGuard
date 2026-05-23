# Track O1-4: Heuristic Ticket Extraction

## Objective
Automatically extract issue ticket identifiers (e.g., Jira, Linear) from the git context to enrich the Intent Capture LLM's understanding and display them in the TUI without relying on brittle external webhooks.

## Requirements
*   **Regex Extraction:** Implement a robust regex scanner for standard ticket formats (e.g., `[A-Z]+-\d+` and `#\d+`).
*   **Context Sources:**
    *   Current branch name (e.g., `feature/ENG-1234-fix-auth`).
    *   `.git/COMMIT_EDITMSG` contents.
    *   Last 10 commit messages in the repository history.
*   **Integration:**
    *   Inject extracted tickets into the `IntentDrafter` context as `related_tickets`.
    *   Display them as active chips in the `RELATED` field of the `ratatui` TUI.

## Definition of Done (DoD)
*   [ ] The extraction logic correctly identifies tickets from branch names and commit messages.
*   [ ] Extracted tickets are successfully passed to the local LLM prompt.
*   [ ] The TUI displays the extracted tickets in the `RELATED` section.
*   [ ] Unit tests cover multiple standard ticket string formats and edge cases.
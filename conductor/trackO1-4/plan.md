# Plan: Track O1-4 (Heuristic Ticket Extraction)

- [ ] 1. Create `src/git/tickets.rs`.
- [ ] 2. Implement the regex extraction logic for `[A-Z]+-\d+` and `#\d+`.
- [ ] 3. Create helper functions to read the current branch name and recent commit history.
- [ ] 4. Wire the extracted tickets into the context generation in `src/ai/intent_drafter.rs`.
- [ ] 5. Wire the extracted tickets into the `IntentState` initialization for the TUI (`src/ui/intent_tui.rs`).
- [ ] 6. Write unit tests for `src/git/tickets.rs`.
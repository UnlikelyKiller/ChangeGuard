# Track U9 Plan: Interactive Category Auto-Correction

- [x] Task U9.1: Implement a fuzzy matching utility in `src/util/` or inside `src/commands/ledger.rs` to map strings to category enums.
- [x] Task U9.2: Integrate the `inquire::Select` prompt inside `ledger start` to let users choose a category when the specified category is invalid and they are in a terminal.
- [x] Task U9.3: Add fallback diagnostic suggestions (like "Did you mean ...?") for non-TTY environments.
- [x] Task U9.4: Add unit and integration tests to verify interactive select behavior and fuzzy corrections.

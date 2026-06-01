# Track U3 Plan: Proactive SQLite PRAGMA Auditing

- [ ] Task U3.1: Run static analysis / ripgrep scans to locate all `PRAGMA` statements across ChangeGuard and AI-Brains source code.
- [ ] Task U3.2: Map pragmas to their expected SQLite return values (setting vs query).
- [ ] Task U3.3: Refactor any settings that accidentally query rows (such as integrity checks) to use `.query_row` or `.query_map`.
- [ ] Task U3.4: Run full test sweeps on both projects to confirm no regression or execute-vs-query mismatch errors occur.

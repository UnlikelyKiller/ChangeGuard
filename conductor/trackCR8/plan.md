# Track CR8 Plan: Escape Symbol Names in Cozo Queries

## Phase 1: Escape Helper Implementation
- [ ] Write a helper function (e.g. `escape_datalog_string`) that replaces single quotes with their escaped counterparts (`\'` or as required by CozoDB string literals) and handles backslashes.
- [ ] Apply this helper to all symbol string interpolations in `src/commands/ask.rs` (such as where the Datalog neighborhood query is dynamically constructed).

## Phase 2: Unit Testing
- [ ] Add unit tests in `src/util/mod.rs` or in query tests verifying that:
  - [ ] Simple strings remain unchanged.
  - [ ] Strings with single quotes are correctly escaped.
  - [ ] Strings with backslashes are correctly escaped.
- [ ] Verify that running a query with an escaped symbol name works end-to-end.

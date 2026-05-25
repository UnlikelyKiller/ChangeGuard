# Track CR8: Escape Symbol Names in Cozo Queries

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
In `src/commands/ask.rs` (and potentially other files executing Datalog queries), symbol names retrieved from the database are interpolated directly into quoted Datalog query literals without escaping. If a symbol name contains single quotes, backslashes, or other special characters, the interpolation breaks the Datalog query syntax, resulting in execution panics or silent query evaluation failures.

## Objective
Implement robust escaping logic for string literals (especially symbol names) before they are interpolated into CozoDB Datalog queries.

## Scope
- Identify all instances in `src/commands/ask.rs` (and adjacent query modules) where symbols or user string values are interpolated into Datalog queries.
- Implement an escaping helper function that safely escapes quotes (`'`), backslashes (`\`), and other Cozo query literal delimiters.
- Ensure that escaped strings are used in all query building steps.

## Success Criteria
- [ ] Symbol names with special characters (e.g. `foo'bar`, `nested\symbol`) are successfully queried in Datalog without syntax errors.
- [ ] No regressions in query parsing or execution for normal symbol names.
- [ ] Unit tests are added to verify string escaping behavior.

## Definition of Done
- [ ] String escaping helper function added to query helpers.
- [ ] Query building in `src/commands/ask.rs` updated to escape symbol names.
- [ ] Unit tests for the escaping helper added.
- [ ] `cargo test` passes.

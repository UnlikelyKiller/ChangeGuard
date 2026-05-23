# Track H6: UX Lifecycle Polish

## Objective
Support entity-based lookups for the `ledger commit` command and general CLI output cleanup.

## Requirements
- **Entity Lookup**: Allow `ledger commit <ENTITY_NAME>` instead of requiring the UUID. If multiple pending transactions have the same entity name, prompt the user or require the ID.
- **Output Tidying**: Remove redundant log messages during standard CLI operations (e.g., duplicate "Success" markers).
- **Progress Clarity**: Improve the wording of the LLM drafting messages to be less technical and more outcome-oriented.

## Definition of Done (DoD)
- [ ] `changeguard ledger commit my_feature` works for unique pending transactions.
- [ ] CLI output for common commands is concise and high-signal.
- [ ] Manual test of the ledger lifecycle feels seamless.

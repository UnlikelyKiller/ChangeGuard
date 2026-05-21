# Track K3: CLI UX Polish (Aliases & Recovery)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
1. **Missing Alias**: Users instinctively type `changeguard status` but the command is `changeguard ledger status`.
2. **Schema Errors**: On `SchemaMismatch`, the CLI simply exits with an error. It should proactively suggest the fix (`update --migrate`).
3. **Subtle Search Modes**: The distinction between BM25 (conceptual) and Trigram (regex) is not intuitive. Unified search or clearer signaling is needed.

## Solution
1. **`status` Alias**: Map `changeguard status` to `execute_ledger(LedgerCommands::Status { ... })`.
2. **Recovery Hints**: Catch `StateError::SchemaMismatch` and `TantivyError::SchemaMismatch` in high-level command executors; wrap them in a `miette` diagnostic with a "help" message suggesting `update --migrate`.
3. **Hybrid/Unified Search**: 
    - Auto-detect regex syntax (e.g. `^`, `.*`, `[`, `]`) and default to `-r` mode.
    - Provide a "Search results for: [Regex/BM25]" header.
    - Implement a `--hybrid` flag that runs both and blends results.

## Definition of Done (DoD)
- [ ] `changeguard status` works and shows ledger state.
- [ ] Schema mismatch errors include the instruction: `Run 'changeguard update --migrate' to recover.`
- [ ] Search query "fn .*_init" automatically triggers regex mode (with a visible notification).
- [ ] CI gate passes.

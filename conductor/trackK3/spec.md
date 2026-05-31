# Track K3: CLI UX Polish (Proactive Recovery & Hybrid Search)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
1. **Missing Alias**: Users instinctively type `changeguard status` but the command is `changeguard ledger status`.
2. **Brittle Schema Errors**: On `SchemaMismatch`, the CLI simply exits. Users are left to guess the recovery command.
3. **Implicit Search Modes**: The distinction between BM25 (conceptual) and Trigram (regex) is a source of confusion.

## Solution
1. **`status` Alias**: Add a top-level `status` command that redirects to `ledger status`.
2. **Proactive Self-Correction**: 
    - Catch `SchemaMismatch` errors in the CLI wrapper.
    - Interactively offer to run `changeguard update --migrate` immediately.
    - If non-interactive (CI), include the exact recovery command in the error message.
3. **Heuristic Hybrid Search**: 
    - Implement a "Search Router" that inspects the query for regex metacharacters (`^`, `.*`, `[`).
    - If regex detected, run `regex_search`. Else run `bm25_search`.
    - Provide a prominent header: `[Search Mode: Regex] query="..."` to clarify the engine used.
    - Add a `--hybrid` flag to run both and deduplicate by path/line.

## Definition of Done (DoD)
- [ ] `changeguard status` shows current transaction and drift state.
- [ ] Artificially breaking the schema results in a prompt: `Schema mismatch detected. Run 'update --migrate' now? [Y/n]`.
- [ ] `changeguard search "pub struct .*"` automatically uses regex mode without the `-r` flag.
- [ ] Search output includes mode signaling.
- [ ] CI gate passes.

# Track X13: `security boundaries` Shows Auth/Authz Summary

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

`changeguard security boundaries` shows a table of security boundary nodes but the output lacks a summary line showing counts by category (principal, action, resource, policy) and a clear `(no data)` hint when the graph has no Cedar-derived entries. The command works but the output context is minimal.

## Problem Statement

After running `changeguard security boundaries`, the output is a bare `comfy-table` with no summary counts, no distinction between policy-derived vs. inferred boundaries, and no empty-state hint when the table is empty. Users cannot tell if the empty table means "no security model" or "Cedar not yet indexed".

## Acceptance Criteria

1. When `security boundaries` has data, the header line shows:
   ```
   Security Boundaries  [3 policies | 12 principals | 8 actions | 4 resources]
   ```
   counts reflect live KG query results.

2. When `security boundaries` returns 0 rows, the empty state shows:
   ```
   No security boundary data found.
   Add Cedar policy files to 'policies/' and run 'changeguard index --analyze-graph'.
   ```
   Yellow message, cyan command.

3. `--json` mode: emit structured JSON with a `meta` object containing the counts.

4. No change to the underlying CozoDB query — output layer only.

## Key Files

- `src/commands/security.rs` — `execute_security_boundaries`
- `src/state/storage_cozo.rs` — CozoDB query helpers

## Definition of Done

- `security boundaries` shows category counts in header when data exists.
- Empty state hint is shown when no data.
- `--json` output includes `meta.counts` object.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

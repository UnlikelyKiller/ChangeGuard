# Track CR7: Robust Global Ask Neighborhood Queries

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
In `src/commands/ask.rs`, the Datalog neighborhood enrichment context is only generated and attached to the query context when the `VectorStore` returns relevant symbols. If the vector query returns no symbols, the command falls back to a legacy chunk-pruner. Under the legacy fallback path, neighborhood extraction is bypassed entirely, meaning the LLM gets context without adjacent neighborhood relations (such as dependencies, callers, etc.) even when some neighborhood info could have been retrieved.

## Objective
Unify the symbol neighborhood extraction logic so that neighbor queries are reliably executed and attached under both the vector retrieval and fallback chunk-pruning branches of the Global Ask flow.

## Scope
- Modify `src/commands/ask.rs` to extract symbol identifiers and call the Datalog neighborhood query helper under the chunk-pruner fallback branch.
- Consolidate symbol-to-neighborhood mapping and query formatting into shared helper functions.
- Ensure unified context construction before sending the payload to the LLM backend.

## Success Criteria
- [ ] Running a global ask query that triggers the chunk-pruner fallback successfully extracts and appends symbol neighborhood context.
- [ ] Datalog neighborhood queries are consistently formatted across both execution paths.
- [ ] Regression/integration tests are added in `tests/cli_ask.rs` to verify context payload integration.

## Definition of Done
- [ ] Unified neighborhood extraction helper implemented in `src/commands/ask.rs`.
- [ ] Legacy fallback branch modified to retrieve symbol neighborhood context.
- [ ] Integration tests added/updated in `tests/cli_ask.rs`.
- [ ] `cargo test` passes.

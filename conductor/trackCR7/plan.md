# Track CR7 Plan: Robust Global Ask Neighborhood Queries

## Phase 1: Implementation
- [ ] Inspect the Global Ask branches inside `src/commands/ask.rs` (vector search vs. fallback chunk-pruner).
- [ ] Extract the neighborhood querying logic from the VectorStore branch into a shared utility function.
- [ ] In the chunk-pruner branch, parse or extract the matched symbols from the retrieved text chunks.
- [ ] Pass these symbols into the shared neighborhood querying function and format the results as enrichment context.

## Phase 2: Testing & Verification
- [ ] Add integration tests in `tests/cli_ask.rs` simulating fallback branch execution (e.g. by querying for terms that trigger the fallback).
- [ ] Verify that the returned context contains both semantic code text and Datalog neighborhood relation reports.
- [ ] Run `cargo test` to verify.

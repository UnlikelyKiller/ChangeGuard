# Track CR7 Plan: Robust Global Ask Neighborhood Queries

## Phase 1: Implementation
- [x] Extracted `fetch_kg_neighborhood()` helper function in `src/commands/ask.rs` that runs Datalog edge queries for a set of symbol names.
- [x] Refactored the existing VectorStore path to use `fetch_kg_neighborhood` (uses CR8 escaping).
- [x] Added KG neighborhood query to the pruner fallback path after chunks are retrieved, extracting symbols from `source` field.

## Phase 2: Testing & Verification
- [x] `cargo test` passes — no regressions.

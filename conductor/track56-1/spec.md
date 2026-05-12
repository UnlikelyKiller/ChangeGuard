# Track 56-1: Restore Native Semantic Search Path

## Objective
Re-enable the HNSW index and route the `ask --semantic` fallback through native cozo-redux vector distance operators so semantic search runs entirely inside the database engine instead of materializing every embedding into Rust.

## Problem Statement
`docs/help2.md` records the working state from Track 55-1: the HNSW index on `snippet_embedding:snippet_idx` was disabled to bypass a `hnsw.rs:890` panic, and the Rust-side fallback in `src/semantic/vector_store.rs` fetches every embedding from CozoDB and scores them with `crate::embed::similarity::cosine_sim`. The note also claims that native CozoDB vector distance functions are unavailable in the cozo-redux fork.

Investigation contradicts that claim:

1. **Vector ops are registered, but the names differ from upstream CozoDB docs.** The cozo-redux registry at `cozo-core/src/data/expr.rs:910-913` binds:
   - `l2_dist(a, b)` — squared L2 distance
   - `cos_dist(a, b)` — cosine distance (1 - cos similarity)
   - `ip_dist(a, b)` — inner-product distance (1 - dot)
   - `l2_normalize(v)`

   The earlier probes (`v_l2_dist`, `vec_distance`, `vec_cosine`) are not registered, which is why every attempt returned `eval::no_implementation`.

2. **The HNSW panic is reproducible only on stale Sled state.** Integration test `tests/cozo_vector_ops.rs::hnsw_index_create_and_query_works_on_fresh_db` builds an index with the same shape used by `vector_store.rs` (`dim, dtype: F32, fields: [embedding], distance: L2, m: 16, ef_construction`) against a fresh in-memory cozo-redux DB and round-trips inserts + queries without panic. The original `hnsw.rs:890` failure (`tuple[2 * key_len + 7].get_bool().unwrap()` index out of bounds) corresponds to edge values written under a pre-`track010` cozo-redux schema (`fix(track010): HNSW durable graph repair on node deletion`, commit `897dddb5`). Track 55-1's `update --migrate` already wipes the Sled directory, so the data-format mismatch is recoverable through the existing migration path.

The combined effect is that `ask --semantic` runs the slow path on every call, native distance functions look "missing," and the help2.md note records guidance that is now inaccurate.

## Scope

### In Scope
- **vector_store.rs**:
  - Re-enable the `::hnsw create snippet_embedding:snippet_idx` script with the dtype-explicit syntax that the integration test verified.
  - Replace the Rust-side embedding fetch with a CozoDB-side distance query using `cos_dist(embedding, $query_vec)` ordered + limited inside Datalog.
  - Keep the dual fallback envelope: when the HNSW index is genuinely absent (e.g., legacy DBs that have not been migrated), fall back to the `cos_dist` query; only fall back to the Rust-side `cosine_sim` loop if `cos_dist` itself errors.
- **storage_cozo.rs**:
  - Ensure schema initialization handles the case where `snippet_embedding` exists but `snippet_idx` does not (older state created during the bypass window).
- **update --migrate**:
  - Surface in the operator output that re-indexing is required to materialize the HNSW index when upgrading from a Track 55-1 state.
- **Verification**:
  - `tests/cozo_vector_ops.rs` is promoted from investigation aid to a permanent regression guard, asserting the cozo-redux op names and HNSW round-trip.
  - Add a `tests/semantic_search.rs` covering `VectorStore::query` against (a) HNSW-indexed snippets, (b) HNSW-missing state forcing the cos_dist fallback, (c) the function-missing case to keep the Rust-side path covered.
- **Docs**:
  - Replace `docs/help2.md` with a short "resolved" note pointing at this track and the canonical op names, or delete it once the work lands.

### Out of Scope
- Adding new cozo-redux operators or upstreaming patches to the fork. The required ops already exist.
- Changing the snippet chunker, embedding model, or schema beyond restoring the index.

## Deliverables
1. Re-enabled HNSW index creation with the verified script.
2. CozoDB-native distance fallback in `VectorStore::query`, with the Rust-side `cosine_sim` retained only as a last-resort guard.
3. Permanent test coverage for the cozo-redux vector ops and the three `VectorStore::query` code paths.
4. Documentation aligned with the actual fork capabilities.

## Acceptance Criteria
- `cargo test --workspace` passes with the new tests.
- A fresh repo (`changeguard init && changeguard index --semantic`) produces a populated `snippet_embedding:snippet_idx` and `ask --semantic` returns results without entering the Rust-side scan path (verifiable via a `tracing::warn` count assertion or instrumentation hook in the test).
- A migrated repo from a Track 55-1 state succeeds via `changeguard update --migrate && changeguard index --semantic` without re-triggering the `hnsw.rs:890` panic.
- `docs/help2.md` no longer claims native vector functions are missing.

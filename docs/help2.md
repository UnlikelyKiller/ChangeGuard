# Resolved: Semantic Search and Cozo-Redux Vector Functions

Superseded by **Track 56-1** (`conductor/track56-1/spec.md`). This note is retained as historical context.

## Summary of Findings

The original investigation reached two incorrect conclusions:

1. **"cozo-redux does not expose vector distance functions."** Incorrect. The fork registers `l2_dist`, `cos_dist`, `ip_dist`, and `l2_normalize` in `cozo-core/src/data/expr.rs` (lines 910-913). The earlier probes (`v_l2_dist`, `vec_distance`, `vec_cosine`) used names that do not exist in either upstream CozoDB or the redux fork, which is why each one returned `eval::no_implementation`.

2. **"HNSW must remain disabled."** Incorrect on fresh state. `tests/cozo_vector_ops.rs` builds an HNSW index with the same shape used by `src/semantic/vector_store.rs` and round-trips create + insert + query without panic. The `hnsw.rs:890` index-out-of-bounds was triggered by stale Sled data written under a pre-`track010` schema (commit `897dddb5 fix(track010): HNSW durable graph repair on node deletion` in the cozo-redux fork). Track 55-1's `update --migrate` already wipes the Sled directory, so the format mismatch is recoverable through normal migration.

## Canonical Operator Names

| Name           | Arity | Semantics                                  |
| -------------- | ----- | ------------------------------------------ |
| `l2_dist`      | 2     | Squared L2 distance, lower is closer       |
| `cos_dist`     | 2     | `1 - cos_sim`, lower is closer             |
| `ip_dist`      | 2     | `1 - dot`, lower is closer                 |
| `l2_normalize` | 1     | Unit-norm projection                       |

## What Track 56-1 Delivers

- Re-enables `::hnsw create snippet_embedding:snippet_idx` with verified syntax.
- Replaces the Rust-side embedding fetch in `VectorStore::query` with a Cozo-native `cos_dist` query.
- Keeps the Rust-side `cosine_sim` loop only as a last-resort guard.
- Promotes `tests/cozo_vector_ops.rs` to a permanent regression test.

Delete this file once Track 56-1 lands.

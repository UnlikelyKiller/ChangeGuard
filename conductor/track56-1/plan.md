# Track 56-1: Restore Native Semantic Search Path — Implementation Plan

## Phase 0: Red Tests (Pre-implementation)
- [x] 1. **Lock in the cozo-redux contract.** — `tests/cozo_vector_ops.rs` promoted to regression guard.
- [x] 2. **Add the failing semantic-path tests.** — `tests/semantic_search.rs` with HNSW happy path, cos_dist fallback, and Rust last-resort tests.

## Phase 1: Schema and Index Restoration
- [x] 1. **`src/semantic/vector_store.rs::setup_schema`** — HNSW create uncommented with `dtype: F32` syntax; idempotent via index existence check.
- [x] 2. **`src/state/storage_cozo.rs`** — Added `get_indices()` wrapper around `::indices <relation>`.

## Phase 2: Query Path Reorganization
- [x] 1. **`src/semantic/vector_store.rs::query`** — Three-tier query path: HNSW (Tier 1) → cos_dist (Tier 2) → Rust cosine_sim (Tier 3). Tiered warn/info messages.

## Phase 3: Migration Surface
- [x] 1. **`src/commands/update.rs`** — Added note that HNSW snippet index will be rebuilt on next `index --semantic`.
- [x] 2. **`src/commands/index.rs`** — Verified by inspection: `VectorStore::new` initializes schema + index before chunk insertion.

## Phase 4: Verification
- [x] 1. CI gate: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace` all pass.
- [x] 2. `cargo install --path .` — binary updated successfully.
- [x] 3. Test coverage: 875 unit tests + all integration tests pass, including `semantic_search` (3 tests) and `cozo_vector_ops` (2 tests).

## Phase 5: Documentation
- [x] 1. `docs/help2.md` — Already updated with resolved note and canonical op names.
- [x] 2. Conductor and plan updated.

## Risk Notes
- **Stale Sled state:** A user upgrading from a Track 55-1 install must run `update --migrate` once before `index --semantic`. The migration already wipes the Sled directory; the operator-facing note added in Phase 3 makes this explicit.
- **Index creation cost:** HNSW build is O(N log N) on insert. The existing chunker volume is small (per-repo snippet counts), so this is acceptable; flag in the spec if benchmarks show otherwise.
- **cosine vs. cos_dist semantics:** `cos_dist = 1 - cos_sim`. The current Rust fallback returns `1 - sim` as a pseudo-distance, so callers already treat the value as a distance — no caller-side changes expected. Audit `src/commands/ask.rs` ranking code once to confirm.

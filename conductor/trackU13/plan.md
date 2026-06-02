# Track U13 Plan: Dynamic HNSW Rebuild Threshold Integration

> **Implementation note:** U11 already shipped `hnsw_rebuild_threshold` plumbing through the config → `VectorStore::new_with_hnsw_threshold` → `HnswRefreshPlan` chain with validation, defaults, and four unit tests. U13 completes the integration surface by adding the parallel `[semantic].concurrency` field (which U14 consumes) and surfacing both in `config verify`.

- [x] Task U13.1: Define and load `semantic.hnsw_rebuild_threshold` in `Config` structures. *(Shipped in U11; verified still present in `src/config/model.rs:227`.)*
- [x] Task U13.2: Reconfigure `HnswRefreshPlan` to accept this threshold limit rather than using a hardcoded constant. *(Shipped in U11; `HnswRefreshPlan::for_batch_with_threshold` at `src/semantic/vector_store.rs:30`.)*
- [x] Task U13.3: Write integration verification tests exercising varying custom thresholds. *(Four `hnsw_refresh_plan_*` tests in `src/semantic/vector_store.rs:535` plus two config-deserialization tests in `src/config/model.rs:1354`.)*
- [x] Task U13.4: Add `semantic.concurrency: Option<usize>` namespace field so the rayon pool size lives next to `hnsw_rebuild_threshold`. *(Added in this pass.)*
- [x] Task U13.5: Add `> 0` validation for `semantic.concurrency` mirroring the existing HNSW check. *(Added in `src/config/validate.rs`.)*
- [x] Task U13.6: Surface semantic settings in `config verify` via `format_semantic_line`. *(Added in `src/commands/config.rs`.)*

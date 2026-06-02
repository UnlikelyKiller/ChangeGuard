# Track U17 Plan: Fix TOML Merge Regression for [semantic] Defaults

- [x] Task U17.1: Write the failing test `test_semantic_partial_section_preserves_defaults` in `src/config/model.rs`.
- [x] Task U17.2: Add `default_hnsw_rebuild_threshold()` helper function.
- [x] Task U17.3: Change `#[serde(default)]` → `#[serde(default = "default_hnsw_rebuild_threshold")]` on `SemanticConfig::hnsw_rebuild_threshold`.
- [x] Task U17.4: Run CI gate; confirm test passes.
- [x] Task U17.5: Manual verification: partial `[semantic]` block keeps `hnsw_rebuild_threshold = 500`.
- [x] Task U17.6: Ledger provenance + commit + push.

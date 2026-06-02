# Track U17 Plan: Fix TOML Merge Regression for [semantic] Defaults

- [ ] Task U17.1: Write the failing test `test_semantic_partial_section_preserves_defaults` in `src/config/model.rs`.
- [ ] Task U17.2: Add `default_hnsw_rebuild_threshold()` helper function.
- [ ] Task U17.3: Change `#[serde(default)]` → `#[serde(default = "default_hnsw_rebuild_threshold")]` on `SemanticConfig::hnsw_rebuild_threshold`.
- [ ] Task U17.4: Run CI gate; confirm test passes.
- [ ] Task U17.5: Manual verification: partial `[semantic]` block keeps `hnsw_rebuild_threshold = 500`.
- [ ] Task U17.6: Ledger provenance + commit + push.

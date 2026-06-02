# Track U16 Plan: Configurable Embed Concurrency Cap

- [ ] Task U16.1: Add `embed_concurrency_cap: Option<usize>` to `SemanticConfig`; update `Default`; add `semantic_embed_concurrency_cap()` accessor.
- [ ] Task U16.2: Add `> 0` validation in `src/config/validate.rs`.
- [ ] Task U16.3: Add default template entry in `src/config/defaults.rs`.
- [ ] Task U16.4: Update the `execute_semantic_index` call site to read the cap and pass it into `ResolveOptions::embed_cap`.
- [ ] Task U16.5: Update `format_semantic_line` in `src/commands/config.rs` to show the cap when non-default.
- [ ] Task U16.6: Write red-phase tests: 1 in model.rs, 1 in validate.rs, 1 in config.rs, extend the existing concurrency test.
- [ ] Task U16.7: Run CI gate.
- [ ] Task U16.8: End-to-end smoke test: set cap=16, run index, verify log shows `embed_concurrency=min(N, 16)`.
- [ ] Task U16.9: Ledger provenance + commit + push.

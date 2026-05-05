## Plan: Track G7 Native Semantic Extraction (De-coupling Part 2)

### Phase 1: Native Semantic Engine
- [ ] **Task 1.1**: Create `src/ai/semantic_extractor.rs` and implement the "Hardened" extraction logic (30k token budget, adaptive recursion).
- [ ] **Task 1.2**: Implement document chunking for Markdown and text files within the Rust core.
- [ ] **Task 1.3**: Wire the semantic extractor into `src/commands/index.rs` using the existing `ai::complete()` interface.

### Phase 2: The Cord-Cut
- [ ] **Task 2.1**: Remove all calls to external Python scripts in `src/commands/index.rs`.
- [ ] **Task 2.2**: Implement a native "Community Detection" wrapper using CozoDB's built-in graph algorithms.
- [ ] **Task 2.3**: Update the CLI to no longer require a `PYTHONPATH` or `graphifyy` installation.

### Phase 3: Final Validation (E2E)
- [ ] **Task 3.1**: Run a full `changeguard index .` on the ChangeGuard repo and verify the graph is identical to the one produced by the hybrid system.
- [ ] **Task 3.2**: Run `changeguard impact` and verify all semantic enrichments are still present.
- [ ] **Task 3.3**: Perform a "Clean Build" of ChangeGuard and verify it runs successfully as a standalone binary on a fresh environment.

### Definition of Done (DoD)
- [x] ChangeGuard is a truly standalone intelligence tool with zero external dependencies.
- [x] Knowledge Graph features are fully integrated into the native Rust binary.
- [x] No more than 4 files modified: `src/ai/semantic_extractor.rs`, `src/ai/mod.rs`, `src/commands/index.rs`, `src/index/mod.rs`.

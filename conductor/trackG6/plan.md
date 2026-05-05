## Plan: Track G6 Native Structural Extraction (De-coupling Part 1)

### Phase 1: Rust Tree-Sitter Port
- [ ] **Task 1.1**: Update `src/languages/rust.rs` to extract "Structural Edges" (imports, trait implementations) using native `tree-sitter-rust`.
- [ ] **Task 1.2**: Update `src/languages/python.rs` to extract module-level imports and method calls.
- [ ] **Task 1.3**: Update `src/languages/typescript.rs` to extract `import` and `require` dependency edges.

### Phase 2: In-Process Edge Discovery
- [ ] **Task 2.1**: Implement `LinkResolver` in `src/index/mod.rs` to resolve relative and absolute import paths to symbol IDs.
- [ ] **Task 2.2**: Update the `index` command to insert these structural edges directly into CozoDB during the AST pass.
- [ ] **Task 2.3**: Implement a "Cycle Detection" health check using CozoDB's recursive Datalog.

### Phase 3: Verification (TDD)
- [ ] **Task 3.1**: Write a test verifying that native Rust extraction finds a cross-file function call and creates the correct edge in CozoDB.
- [ ] **Task 3.2**: Compare native extraction results with `graphifyy` output to ensure no regression in edge density.
- [ ] **Task 3.3**: Verify that complex import patterns (e.g., `use crate::...`, `from . import ...`) are correctly resolved.

### Definition of Done (DoD)
- [x] Structural code relationships are extracted natively in Rust.
- [x] No external Python dependency is required for the AST/Structural graph.
- [x] No more than 4 files modified: `src/languages/rust.rs`, `src/languages/python.rs`, `src/languages/typescript.rs`, `src/index/mod.rs`.

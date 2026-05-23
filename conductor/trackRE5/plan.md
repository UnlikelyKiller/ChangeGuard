# Plan: Track RE5 (Segment `src/index/languages/rust.rs`)

- [ ] 1. Create the directory `src/index/languages/rust/`.
- [ ] 2. Create `imports.rs`, `types.rs`, `functions.rs`, and `traits.rs` within that directory.
- [ ] 3. Move the `tree-sitter` query strings and result mapping logic for each category to the new files.
- [ ] 4. Refactor the `extract_symbols` implementation in `src/index/languages/rust.rs` to delegate to these sub-modules.
- [ ] 5. Consolidate common AST traversal utilities into a shared helper in the `rust/` module.
- [ ] 6. Run symbol extraction tests against diverse Rust samples to ensure accuracy.

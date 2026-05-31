# Track RE5: Segment `src/index/languages/rust.rs`

## Objective
Split the monolithic Rust AST parser into modular components based on symbol type (Types, Functions, Imports, Traits).

## Requirements
- **Module Separation**: Create `src/index/languages/rust/types.rs`, `functions.rs`, `imports.rs`, and `traits.rs`.
- **Query Management**: Move the specific `tree-sitter` S-expression queries to their respective modules.
- **Entrypoint**: Use `src/index/languages/rust.rs` as the thin wrapper and orchestrator for these sub-parsers.

## Definition of Done (DoD)
- [ ] `src/index/languages/rust.rs` is reduced to < 300 lines.
- [ ] Symbol extraction logic is cleanly segmented.
- [ ] No regression in symbol detection for Rust repositories.
- [ ] All 897 tests pass.

# Specification: Track G6 Native Structural Extraction (De-coupling Part 1)

## Goal
Port the AST-based "Link Discovery" logic from the external `graphifyy` package into the ChangeGuard Rust core, enabling native structural graph extraction.

## Context
This is the first step in "Cord-Cutting." By extracting structural edges (imports, calls) natively in Rust, we remove the need for Python during the most frequent indexing operations.

## Technical Details

### 1. Language Handlers Extension
Extend the `LanguageHandler` trait (and implementations in `src/languages/*.rs`) to include an `extract_edges` method:
- **Rust**: Find `use` statements and fully-qualified calls.
- **Python**: Find `import` and `from ... import` statements.
- **TypeScript**: Find `import` and `require()` calls.

### 2. Link Resolver (`src/index/mod.rs`)
Implement a `LinkResolver` that:
1.  Takes a raw import string (e.g., `crate::ledger::db`).
2.  Uses the `project_topology` and `project_symbols` to find the target symbol ID.
3.  Handles relative vs absolute paths and language-specific aliasing.

### 3. Direct Cozo Insertion
Modify the `index` command to insert these discovered edges directly into the `edge` relation in CozoDB with `confidence = 1.0` (EXTRACTED).

## TDD Requirements
1.  **Import Resolution**: Test that a Rust `use` statement correctly resolves to a symbol in another file.
2.  **Edge Creation**: Verify that indexing a 2-file project creates a `calls` or `imports` edge between them in CozoDB.
3.  **Parity Test**: Ensure that the number of structural edges found by the Rust engine matches the number found by `graphifyy` for a test corpus.

## Definition of Done
- [ ] Structural edge extraction implemented for Rust, Python, and TypeScript.
- [ ] `LinkResolver` correctly resolves cross-file dependencies.
- [ ] Structural graph works without any external Python dependency.
- [ ] No more than 4 files modified: `src/languages/rust.rs`, `src/languages/python.rs`, `src/languages/typescript.rs`, `src/index/mod.rs`.

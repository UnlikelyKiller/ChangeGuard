# Technical Specification: Track 7 - Language-Aware Symbol Extraction

## Overview
Phase 8 objective: Extract symbol metadata from changed files (Rust, TypeScript, Python) using `tree-sitter` to enrich impact packets. The extraction must gracefully tolerate syntax errors, incomplete files, and unknown file types without crashing the scan process.

## Architecture

### 1. Data Models (`src/index/symbols.rs` & `src/impact/packet.rs`)
Define a universal representation for symbols extracted from supported languages:
- `SymbolKind`: Enum representing the type of symbol (e.g., `Function`, `Class`, `Struct`, `Variable`, `Interface`, `Type`).
- `Symbol`: Struct containing the symbol's name, kind, and a boolean `is_export` or `is_public`.

Modify `ChangedFile` in `src/impact/packet.rs` to include an optional field:
```rust
pub symbols: Option<Vec<Symbol>>,
```
(Optional so that non-supported languages or failed parses can gracefully hold `None`).

### 2. Language Dispatch (`src/index/languages/mod.rs`)
A central dispatcher that inspects a file's extension and delegates to the appropriate language parser.
- Extension mapping:
  - `.rs` -> Rust
  - `.ts`, `.tsx`, `.js`, `.jsx` -> TypeScript/JavaScript
  - `.py` -> Python

### 3. Tree-Sitter Integration
Leverage the pinned tree-sitter crates in `Cargo.toml`:
- `tree-sitter` (0.26.8)
- `tree-sitter-rust` (0.24.2)
- `tree-sitter-typescript` (0.23.2)
- `tree-sitter-python` (0.25.0)

For each language, create a module (`rust.rs`, `typescript.rs`, `python.rs`) that:
1. Instantiates a `tree_sitter::Parser` and sets the corresponding language.
2. Parses the file content.
3. Uses `tree_sitter::Query` to extract specific symbol types and their visibility.

#### Rust Extraction Strategy
- Extract `fn`, `struct`, `enum`, `trait`, `type`.
- Determine public visibility by checking for the `pub` keyword node modifying the declaration.

#### TypeScript/JavaScript Extraction Strategy
- Extract `function`, `class`, `interface`, `type_alias_declaration`, `export_statement`.
- Determine public visibility by checking if the symbol is exported (`export` modifier or `export { ... }`). Use the `typescript` language parser from `tree-sitter-typescript`.

#### Python Extraction Strategy
- Extract `def` (functions), `class` (classes).
- Python has no strict `public`/`private` modifiers, but conventionally symbols starting with `_` are private.

### 4. Error Handling & Resilience
- **Parsing failures:** Incomplete files or syntax errors must *not* crash the scan. Tree-Sitter is robust to syntax errors and will build partial ASTs. If a file cannot be processed at all (e.g., bad encoding or internal parser panic), catch and log the warning via `tracing::warn!` and return `None` for the symbols list.
- **Idiomatic Rust:** Use `miette` for rich error contexts and return `miette::Result<Option<Vec<Symbol>>>` from parser functions, allowing the caller to decide whether to surface the error or swallow it.
- **Memory/Performance:** Read file contents as efficiently as possible and drop AST structures after extracting metadata to avoid unbounded memory growth during batch scans.
# Track 54-1: Native Code-Aware Tokenization (Tree-Sitter FTS Integration)

## Objective
Enhance the precision of Full-Text Search (FTS) within the Knowledge Graph by replacing generic text tokenizers with a native Tree-Sitter based tokenizer implementation in the `cozo-redux` engine.

## Problem Statement
Current FTS tokenizers (e.g., Tantivy defaults or generic regex) treat source code as plain natural language. This leads to several issues:
1.  **Symbol Noise**: Searching for `main` returns results for the `main` function, but also every mention of "main" in comments, strings, or unrelated variable names.
2.  **Structural Ignorance**: Operators, macros, and complex symbols (e.g., `Option<Result<T, E>>`) are often split or tokenized inconsistently, making precise queries difficult.
3.  **Language Blindness**: Tokenization rules for Rust differ from TypeScript or Go, but generic tokenizers apply a "one size fits all" approach.

## Scope
-   **CozoDB Fork (Core)**:
    -   Integrate `tree-sitter` as a native dependency in the `cozo-core` FTS module.
    -   Implement a `CodeTokenizer` that uses Tree-Sitter grammars to identify and weight different AST node types (Definitions > References > Comments).
    -   Expose a configuration interface to specify the language per-relation or per-row.
-   **ChangeGuard Integration**:
    -   Update `changeguard index` to leverage the new code-aware FTS mode.
    -   Implement a fallback mechanism for unsupported languages.
    -   Update `changeguard search` to support AST-based filters (e.g., `search --kind function hello`).

## Deliverables
-   Updated `cozo-redux` fork with `CodeTokenizer`.
-   New `index --fts-mode code` capability in ChangeGuard.
-   Unit tests verifying symbol-vs-comment search precision.

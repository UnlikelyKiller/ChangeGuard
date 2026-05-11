# Track 54-1: Native Code-Aware Tokenization - Implementation Plan

## Phase 1: CozoDB Engine Implementation (Upstream Fork)
1.  **Grammar Integration**:
    - Add `tree-sitter-rust`, `tree-sitter-typescript`, and `tree-sitter-go` to the `cozo-core` dependencies.
    - Implement a `GrammarRegistry` to manage language-specific parsers.
2.  **Tokenizer Development**:
    - Implement the `CodeTokenizer` trait.
    - Logic: Parse the input text using Tree-Sitter, iterate through the AST, and yield tokens only for "meaningful" nodes (identifiers, type names, macro invocations).
    - Implement "Semantic Weighting": Increase the search score for tokens found in definition nodes vs. those found in string literals.
3.  **FTS API Extension**:
    - Update the `:put node` and `:put edge` logic to accept an optional `fts_mode` parameter.

## Phase 2: ChangeGuard Integration
1.  **Index Orchestration**:
    - Update `src/index/orchestrator.rs` to detect the source file language and pass it to the Knowledge Graph during ingestion.
2.  **CLI Updates**:
    - Add `--fts-mode [generic|code]` to the `index` command (defaulting to `code` for supported extensions).
3.  **Search Refinement**:
    - Update `src/commands/search.rs` to allow filtering by AST node type if the `code` tokenizer was used.

## Phase 3: Verification
1.  **Precision Benchmark**:
    - Index a repository containing a function `fn hello()` and a comment `// Just saying hello`.
    - Verify that `search hello` ranks the function definition significantly higher than the comment.
2.  **Language Test**:
    - Verify correct tokenization for different languages in the same federated Knowledge Graph.
3.  **Performance Check**:
    - Ensure that Tree-Sitter parsing during indexing does not introduce more than 10% latency overhead compared to generic regex tokenization.

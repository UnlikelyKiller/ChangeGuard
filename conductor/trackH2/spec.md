# Track H2: Advanced Snippet Ingestion

## Objective
Transition from whole-file semantic indexing to symbol-aware snippet ingestion to improve "Ask" and "Search" precision.

## Requirements
- **Parser Integration**: Leverage the new modular Rust parser (`src/index/languages/rust/symbols.rs`) during the semantic indexing phase.
- **Granular Chunking**: Instead of one snippet per file, create one snippet per significant symbol (function, struct, impl block).
- **Metadata Enrichment**: Include the symbol's qualified name and kind in the embedding metadata for better attribution in search results.
- **Incremental Consistency**: Ensure that when a file is re-indexed, its old symbol-snippets are properly removed from the vector store.

## Definition of Done (DoD)
- [ ] `changeguard index --semantic` creates multiple snippets for multi-symbol files.
- [ ] Semantic search results identify the specific symbol found, not just the file path.
- [ ] Performance of indexing remains within acceptable limits for 100+ files.

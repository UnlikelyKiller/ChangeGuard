# Plan: Track H2 (Advanced Snippet Ingestion)

- [ ] 1. Update `SnippetIngester` to take an optional list of extracted symbols.
- [ ] 2. Refactor `src/semantic/chunker.rs` to break content by byte ranges provided by the parser.
- [ ] 3. Modify the semantic indexing loop in `src/commands/index.rs` to pass symbols to the ingester.
- [ ] 4. Update the CozoDB Datalog queries for semantic search to return symbol-level metadata.
- [ ] 5. Verify that `search --semantic` identifies the correct function within a file.

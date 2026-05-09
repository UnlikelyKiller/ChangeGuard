# Implementation Plan - Track S1: High-Performance Global Code Search

## Goal
Implement sub-millisecond trigram-based regex search across the federated codebase, combining BM25-ranked full-text search with a high-performance trigram pre-filter.

## Proposed Changes

### 1. Tantivy Ranking Engine [src/search/tantivy_engine.rs] [NEW]
- Define Tantivy schema for code documents (path, content, language, line_count).
- Implement `TantivySearchEngine` with `index_repo()` and `search(query: &str) -> Result<Vec<SearchResult>>`.
- Configure memory-mapped segment storage and faceted search.

### 2. Trigram Filter & Regex Pipeline [src/search/trigram.rs] [NEW]
- Implement `TrigramIndex` builder that extracts 3-grams from file content.
- Add `regex_to_trigrams(pattern: &str) -> Option<Vec<String>>` to derive required trigrams from a regex.
- Implement candidate-file intersection logic.

### 3. Regex Execution Layer [src/search/regex_filter.rs] [NEW]
- Build `RegexFilter` that receives candidate file sets from the trigram index.
- Run actual regex on candidate files only, with line-length and timeout guards to prevent catastrophic backtracking.
- Return syntax-highlighted snippets.

### 4. Streaming Indexer [src/search/stream_indexer.rs] [NEW]
- Implement producer-consumer model using `crossbeam` channels.
- Producer walks the repo respecting `.gitignore` and `SUPPORTED_EXTENSIONS`.
- Consumers feed documents to Tantivy and the trigram index incrementally.

### 5. Encoding & Edge-Case Hardening [src/search/encoding.rs] [NEW]
- Normalize source files to UTF-8 using `encoding_rs`; strip non-printable control characters.
- Skip or truncate files >1MB.
- Detect and skip minified/long lines (>10k chars).

### 6. CLI Integration [src/commands/search.rs]
- Add `changeguard search <query>` for ranked search.
- Add `--regex <pattern>` for trigram-accelerated regex search.
- Wire search engines into the CLI command dispatcher.

## Verification Plan

### Automated Tests
- `cargo test`: All existing tests pass.
- Unit tests for trigram extraction and regex-to-trigram translation.
- Unit tests for Tantivy indexing and querying on a synthetic repo fixture.
- Integration test in `tests/search_performance.rs` asserting <100ms search on a 1M-line fixture.

### Manual Verification
- Run `changeguard search` and `changeguard search --regex` on the ChangeGuard repo itself and verify result quality and performance.

## Definition of Done (DoD)

- [ ] **Performance**: Global search completes in <100ms on 1M+ lines of code.
- [ ] **Trigram Filter**: `regex_to_trigrams` correctly identifies candidates and eliminates exhaustive scans for common patterns.
- [ ] **Ranking**: Tantivy BM25 results are returned with syntax-highlighted snippets.
- [ ] **Edge Cases**: Large files, long lines, and non-UTF-8 encodings are handled gracefully without panic or mis-encoding.
- [ ] **Test Coverage**: Unit tests for trigram, Tantivy, and regex filter; integration test for performance gate.
- [ ] **Zero Regression**: All existing `cargo test` suites pass unchanged.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, and `cargo test` pass with zero warnings.

# Track S1: High-Performance Global Code Search

## Overview
Implement a search engine that combines ranked full-text search with extremely fast regex filtering via trigram-based pre-filtering. This provides sub-millisecond search across the federated codebase, leveraging memory-mapped indices and streaming indexing to handle massive monorepos.

## Objectives
- Integrate `tantivy` crate for BM25-ranked full-text indexing with schema-based segment storage.
- Implement a Trigram-based candidate filter (zoekt-style) for high-speed regex pre-filtering before exhaustive regex scans.
- Use memory-mapped files for indices to support codebases exceeding available RAM.
- Build a streaming indexer using a producer-consumer model to prevent memory spikes during ingestion.
- Handle edge cases: large files (>1MB), long lines/minified JS, and non-UTF-8 encodings.

## Success Criteria
- `changeguard search <query>` returns ranked results with syntax-highlighted snippets in <100ms on 1M+ lines.
- `changeguard search --regex <pattern>` leverages trigram pre-filtering and avoids catastrophic backtracking.
- Automatic exclusions inherit `.gitignore` and `SUPPORTED_EXTENSIONS` from the core indexer.
- Indexing completes without memory spikes on repos with >100k files.

## Architecture
- `src/search/mod.rs`: Core search module and public API.
- `src/search/tantivy_engine.rs`: Tantivy schema, indexing, and BM25 query execution.
- `src/search/trigram.rs`: Trigram index generation and regex-to-trigram translation.
- `src/search/regex_filter.rs`: Candidate filtering and regex execution pipeline.
- `src/search/stream_indexer.rs`: Producer-consumer streaming indexer.
- `src/search/encoding.rs`: UTF-8 normalization via `encoding_rs` and control-character stripping.

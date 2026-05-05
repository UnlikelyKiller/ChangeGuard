# Milestone S: Global Intelligence & Precision Search

This milestone evolves ChangeGuard into a comprehensive codebase research and precision-indexing engine. It targets sub-millisecond search, compiler-grade navigation, and semantic discovery, ensuring resilient operations across massive, multi-language monorepos.

---

## Track S1: Hybrid High-Performance Code Search

**Goal**: Implement a search engine that combines ranked full-text search with extremely fast regex filtering.

### Implementation Strategy
*   **Ranking Engine (Tantivy)**: Integrate the `tantivy` crate for schema-based indexing. This provides BM25 relevance scoring, faceted search, and efficient segment-based storage.
*   **Regex Engine (Trigram Filter)**: Implement a Trigram-based candidate filter (similar to `zoekt`). Trigrams enable high-speed regex pre-filtering before passing candidate files to the regex engine, avoiding exhaustive scans.
*   **Memory Efficiency**: Use memory-mapped files for indices to handle codebases that exceed available RAM.
*   **Streaming Indexer**: Use a producer-consumer model (via `crossbeam` or `tokio-mpsc`) to index files as they are read, preventing memory spikes.

### Edge Case Handling
*   **Large Files**: Cap indexable file size (e.g., 1MB) and truncate longer files to prevent indexing artifacts or logs.
*   **Long Lines**: Detect and skip minified JS or massive data lines that crash regex engines (the "Catastrophic Backtracking" problem).
*   **Encodings**: Use `encoding_rs` to normalize all source files to UTF-8; ignore or strip non-printable control characters.

### Key Deliverables
*   `changeguard search <query>`: Ranked search results with syntax-highlighted snippets.
*   `--regex`: High-speed regex search powered by the Trigram pre-filter.
*   **Automatic Exclusions**: Inherit `.gitignore` and `SUPPORTED_EXTENSIONS` from the core indexer.

---

## Track S2: SCIP-Based Precision Indexing

**Goal**: Achieve "compiler-perfect" navigation by ingesting external indices from language-specific indexers.

### Implementation Strategy
*   **Protobuf Ingestion**: Use the `scip` crate to parse SCIP (Symbolic Code Index Protocol) Protobuf files.
*   **Unified Symbol Table**: Map SCIP symbols to ChangeGuard's `project_symbols` in SQLite/CozoDB. This ensures that `changeguard impact` uses the same symbol IDs whether derived from Tree-sitter or SCIP.
*   **Incremental Ingestion**: Only re-process SCIP indices if the underlying index file has changed (checked via BLAKE3 hash).

### Hardening & Resilience
*   **Partial Indexing**: Gracefully handle SCIP indices that only cover a subset of the repo. Use Tree-sitter as a fallback for un-indexed modules.
*   **Stale Index Detection**: Flag a warning if the SCIP index refers to file hashes that no longer match the current working tree.
*   **Path Normalization**: Ensure SCIP's relative paths are correctly lexicalized to the repo root to prevent cross-platform mismatch.

### Key Deliverables
*   `changeguard index --scip <path>`: Ingest an external index.
*   **Precision Navigation**: Multi-language Go-to-Definition and Find-References in `viz` and `impact` that handle complex cross-file dependencies (e.g., Trait/Interface implementations).

---

## Track S3: Semantic Discovery & "Concept" Search

**Goal**: Enable conceptual retrieval by embedding code logic blocks into vector space.

### Implementation Strategy
*   **Tree-sitter Block Chunking**: Don't chunk by line or character. Chunk by **AST Logical Blocks**:
    *   Functions (Signature + Docstring + Body)
    *   Classes/Structs (Definition + Methods)
    *   Module-level constants and exported types.
*   **Local Embedding (llama-server)**: Use an OpenAI-compatible endpoint to fetch embeddings using a lightweight local model (e.g., `nomic-embed-text` or `bge-small-en`).
*   **Vector Querying**: Utilize CozoDB's native vector similarity functions (`hnsw` index) for Top-K retrieval.

### Edge Case Handling
*   **Massive Functions**: Split functions larger than the model's context window (e.g., 512 tokens) using overlapping windows to preserve context.
*   **Comment vs Code**: Weight the embeddings to prioritize docstrings and comments, as they contain the "intent" that semantic search usually targets.

### Key Deliverables
*   `changeguard ask --semantic <query>`: Find code by purpose (e.g., "Where is the retry logic for the database?").
*   **Semantic Hotspots**: Identify "redundant logic" where multiple functions share high semantic similarity (>0.9 cosine similarity).

---

## Track S4: Verification & TDD (Hardening)

**Goal**: Ensure the search engine remains performant and accurate during rapid development.

### Test Tracks
*   **Performance Gate**: Automated benchmark in CI that fails if `changeguard search` takes >100ms on the ChangeGuard repo itself.
*   **Accuracy Suite**: A "Gold Set" of search queries and expected results (True Positives/Negatives) to prevent regression in the Trigram filter.
*   **Encoding stress-test**: A dedicated test fixture with UTF-16, Shift-JIS, and malformed UTF-8 files to ensure indexer stability.

---

## Definition of Done (DoD)
1. [ ] Global search completes in <100ms on 1M+ lines of code.
2. [ ] SCIP symbols are successfully merged with Tree-sitter symbols in CozoDB.
3. [ ] `changeguard ask --semantic` can retrieve a function from a natural language query with >80% accuracy.
4. [ ] All search features work offline using local embedding and ranking engines.
5. [ ] The `viz` command includes a "Semantic Search" search bar.

# Implementation Plan - Track S3: Semantic Discovery for "Code Snippets"

## Goal
Implement local vector embedding and search for fine-grained code logic blocks, enabling natural-language conceptual retrieval and redundant-logic detection.

## Proposed Changes

### 1. AST-Based Chunking [src/semantic/chunker.rs] [NEW]
- Implement `AstChunker` using Tree-sitter to extract logical blocks:
  - Functions (signature + docstring + body).
  - Classes/Structs (definition + methods).
  - Module-level constants and exported types.
- Split functions exceeding the model's context window (e.g., 512 tokens) into overlapping windows.
- Skip generated/minified blocks.

### 2. Local Embedding Client [src/semantic/embedder.rs] [NEW]
- Implement `LocalEmbedder` that calls an OpenAI-compatible local endpoint (`llama-server`).
- Default to a lightweight model (e.g., `nomic-embed-text` or `bge-small-en`).
- Cache embeddings on disk to avoid re-embedding unchanged blocks (keyed by BLAKE3 of block content).

### 3. Vector Storage & Querying [src/semantic/vector_store.rs] [NEW]
- Store embedding vectors in CozoDB with an `hnsw` approximate-nearest-neighbor index.
- Implement `SemanticIndex::index_chunks(chunks: Vec<AstChunk>) -> Result<()>`.
- Implement `SemanticIndex::query(query_embedding: Vec<f32>, k: usize) -> Result<Vec<SemanticResult>>`.

### 4. Semantic Hotspot Detection [src/semantic/hotspots.rs] [NEW]
- Compute pairwise cosine similarity across indexed function embeddings.
- Flag clusters with similarity >0.9 as potential redundant logic.
- Surface hotspots in `changeguard impact` warnings or a dedicated report.

### 5. Weighted Embedding Strategy [src/semantic/chunker.rs]
- Prioritize docstrings and comments during chunk text assembly so the embedding captures intent, not just implementation.
- Strip boilerplate headers if they do not contain semantic signal.

### 6. CLI & Viz Integration [src/commands/ask.rs] [NEW/UPDATE]
- Add `changeguard ask --semantic <query>` command.
- Wire natural-language query -> embedder -> vector_store -> ranked results.
- Add a "Semantic Search" search bar to the `viz` HTML/frontend output.

## Verification Plan

### Automated Tests
- `cargo test`: All existing tests pass.
- Unit tests for `AstChunker` on synthetic Rust/TypeScript/Python fixtures.
- Unit tests for `LocalEmbedder` mock endpoint interaction.
- Integration test in `tests/semantic_accuracy.rs` with a gold set of queries asserting >80% top-1 accuracy.

### Manual Verification
- Run `changeguard ask --semantic` with natural-language queries on the ChangeGuard repo and inspect result relevance.
- Verify `viz` semantic search bar renders and returns results.

## Definition of Done (DoD)

- [ ] **Chunking**: AST-based logical block chunker runs without panic on all supported languages.
- [ ] **Embeddings**: Local embedder produces vectors and caches them correctly.
- [ ] **Search Accuracy**: `changeguard ask --semantic` achieves >80% top-1 accuracy on the gold query set.
- [ ] **Offline Operation**: All semantic search features work without network access to external APIs.
- [ ] **Hotspots**: Redundant-logic detection flags clusters with cosine similarity >0.9.
- [ ] **Viz Integration**: The `viz` command includes a working "Semantic Search" search bar.
- [ ] **Test Coverage**: Unit tests for chunker, embedder, and vector store; integration test for accuracy.
- [ ] **Zero Regression**: All existing `cargo test` suites pass unchanged.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, and `cargo test` pass with zero warnings.

# Track S3: Semantic Discovery for "Code Snippets"

## Overview
Enable conceptual retrieval of fine-grained code logic blocks by embedding them into vector space using a local model. Tree-sitter AST-based chunking preserves semantic coherence, and CozoDB's HNSW index provides fast Top-K vector similarity search for natural-language queries.

## Objectives
- Chunk code by AST logical blocks (functions, classes/structs, module-level constants) rather than by line or character.
- Generate embeddings via a local OpenAI-compatible endpoint (e.g., `llama-server` with `nomic-embed-text` or `bge-small-en`).
- Store vectors in CozoDB with an `hnsw` index for approximate nearest-neighbor search.
- Handle massive functions by splitting them into overlapping windows.
- Weight embeddings toward docstrings and comments to capture intent.

## Success Criteria
- `changeguard ask --semantic <query>` returns relevant functions/classes from a natural-language query with >80% top-1 accuracy on a gold set.
- Semantic search operates fully offline using local embedding and ranking engines.
- "Semantic Hotspots" can flag redundant logic where multiple functions share cosine similarity >0.9.
- The `viz` command includes a "Semantic Search" search bar.

## Architecture
- `src/semantic/mod.rs`: Semantic search module and public API.
- `src/semantic/chunker.rs`: Tree-sitter AST-based logical block chunker.
- `src/semantic/embedder.rs`: Local embedding client (OpenAI-compatible HTTP endpoint).
- `src/semantic/vector_store.rs`: CozoDB vector insertion and HNSW-indexed querying.
- `src/semantic/hotspots.rs`: Redundant-logic detection via cosine similarity clustering.
- `src/commands/ask.rs`: CLI wrapper for `ask --semantic`.

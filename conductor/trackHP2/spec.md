# Track HP2: Parallelized AST Chunk Ingestion & Embedding Generation

## Objective
Accelerate full codebase indexing by parallelizing tree-sitter AST snippet extraction and LLM embedding requests.

## Requirements
- **Parallel AST Snippet Extraction**: Use `rayon` or a thread pool to parse multiple files in parallel during full codebase sweeps, extracting Tree-Sitter AST chunks concurrently.
- **Parallel Embedding Ingestion**: Batch and send embedding API queries concurrently across multiple threads to maximize local model GPU utilization or Gemini API throughput.
- **Thread Safety**: Ensure CozoDB connection handoffs and vector store ingestion queues are fully thread-safe.

## Definition of Done (DoD)
- [ ] Codebase indexing execution time is reduced by at least 40% on multi-core systems.
- [ ] Concurrency levels can be limited/configured via CLI arguments or config properties.
- [ ] Full indexing passes all integration and consistency checks without race conditions.

# Track U7 Spec: High-Performance Parallel Indexing

## Background
Full semantic indexing can take 20+ minutes for medium-sized repositories. The current implementation performs Tree-Sitter parsing and embedding generation in a mostly sequential or batch-sequential manner.

## Objective
Dramatically reduce indexing time by parallelizing the parsing and embedding phases.

## Proposed Design
* Use `rayon::iter::ParallelIterator` to process files in parallel.
* Parsing Phase: Parallelize `ProjectIndexer::index_all` using `par_iter()`. Ensure a separate parser instance is used per thread as per best practices.
* Embedding Phase: Use batching (max 8-16) to avoid overloading the local model server, but parallelize the preparation of these batches.
* Performance Target: 2-3x speedup on multi-core systems for the parsing phase.
* Add progress bar support that accurately reflects multi-threaded progress using `indicatif`.

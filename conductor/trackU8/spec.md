# Track U8 Spec: HNSW Build Speed Optimization

## Background
Currently, rebuilding the HNSW semantic index when many chunks are modified/ingested takes significant time (~3 minutes for 300+ items). This is because the index is dropped and built from scratch.

## Objective
Optimize HNSW index construction during semantic indexing by implementing caching of the index or leveraging incremental updates / caching vectors to disk to avoid full index rebuilding.

## Proposed Design
* Avoid dropping HNSW index during incremental updates if the batch size is below a configurable threshold, or optimize CozoDB batch parameters to build HNSW faster.
* Cache the generated vectors and indexes.
* Investigate using Sled's cache size configurations or CozoDB HNSW options to tune performance on larger chunk sets.

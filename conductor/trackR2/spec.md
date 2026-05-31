# Specification: Search Noise Reduction & Scoped Viz

## Objective
Improve search precision by implementing strict trigram pre-filtering in BM25 searches and enhance visualization utility by adding scoping flags.

## Requirements
### Search Noise Reduction
- File: `src/search/tantivy_engine.rs` (and potentially `src/search/trigram.rs`).
- Goal: Filter out common-term noise by requiring valid trigram matches before executing full BM25 scoring.
### Scoped Viz
- File: `src/commands/viz.rs` and `src/commands/viz_server.rs`.
- Add CLI flags:
  - `--limit <usize>`: Max nodes to visualize.
  - `--depth <usize>`: Max traversal depth from a target.
  - `--entity <String>`: Specific entity/node to center the visualization on.
- Plumb these parameters into the viz data generator (CozoDB query/graph builder).

## Architecture
- Search: Add a trigram index check or query layer before the Tantivy search phase.
- Viz: Extend `clap` structs in `viz.rs`. Update the knowledge graph query to respect the limits.
# Track U13 Spec: Dynamic HNSW Rebuild Threshold Integration

## Background
We previously designed Track U11 to parameterize the HNSW rebuild threshold. This track completes that scope by fully integrating and exposing it under a `[semantic]` namespace block (e.g. `semantic.hnsw_rebuild_threshold = 500`) in `config.toml`, and reading it inside `VectorStore::index_chunks`.

## Objective
Provide user configurability for the exact ingestion batch size that determines when to drop and rebuild the HNSW index versus appending to it incrementally.

## Proposed Design
* Map `semantic.hnsw_rebuild_threshold` (default 500) inside `src/config/model.rs`.
* Use the parameter inside the dynamic calculation phase in `VectorStore`.

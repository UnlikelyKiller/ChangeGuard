# Track U11 Spec: Dynamic HNSW Rebuild Threshold Configuration

## Background
Currently, the threshold for dropping and rebuilding the HNSW index during vector store operations is hardcoded to `500` inside `HnswRefreshPlan` in `src/semantic/vector_store.rs`. 

## Objective
Make this threshold dynamic and configurable by the user via the `config.toml` file under a configuration key such as `semantic.hnsw_rebuild_threshold` (defaulting to `500`).

## Proposed Design
* Introduce `hnsw_rebuild_threshold: Option<usize>` in the semantic configuration.
* Pass the configured value into the `VectorStore` structure or resolve it dynamically when evaluating the ingestion batch size.
* Implement tests verifying that the threshold is read from user configuration and respected.

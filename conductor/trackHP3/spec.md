# Track HP3: Cached Vector Nodes & Incremental HNSW Appends

## Objective
Optimize incremental index updates by persisting vector store HNSW configurations to disk, enabling fast append-only writes instead of rebuilding the entire vector store index.

## Requirements
- **Disk-Backed HNSW Configurations**: Save the state of the HNSW index graph (nodes, links) directly into `ledger.cozo` or cache files under `.changeguard/state/`.
- **Append-Only Graph Insertion**: Implement incremental appends for newly modified/added files, inserting only the new vector nodes into the HNSW graph rather than rebuilding all nodes.
- **Node Pruning/Wiping**: Implement clean node removal from the HNSW graph when files are deleted or modified.

## Definition of Done (DoD)
- [ ] Incremental indexes (`index --incremental`) complete in under 2 seconds when only a few files are changed.
- [ ] Direct HNSW searches yield correct and identical recall scores compared to a full rebuild.
- [ ] Verification tests for graph persistence and incremental indexing pass cleanly.

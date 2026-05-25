# Track HP4: Snippet Ingestion Progress & HNSW Build UX

## Objective
Provide real-time developer feedback during long-running HNSW index builds on large codebase indexing runs.

## Requirements
- **Progress Tracking**: Render a dynamic terminal progress bar (using the existing `indicatif` library in ChangeGuard) during the HNSW indexing and vector graph build phases.
- **Verbose Log Reporting**: Provide clear summaries of elapsed time, number of ingested elements, and CPU/memory utilization if verbose logging is enabled.

## Definition of Done (DoD)
- [ ] Running `changeguard index --semantic` displays a visual progress bar indicating HNSW build progression.
- [ ] No layout glitches or terminal character corruption occurs on Windows PowerShell/CMD.

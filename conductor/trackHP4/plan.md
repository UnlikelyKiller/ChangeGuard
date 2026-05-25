# Plan: Track HP4 (Snippet Ingestion Progress & HNSW Build UX)

- [ ] 1. Modify `src/semantic/vector_store.rs` to expose progression metrics (elements inserted, links constructed) during HNSW construction.
- [ ] 2. Update `src/commands/index.rs` to initialize and drive an `indicatif::ProgressBar` styled consistently with the rest of the ChangeGuard UI.
- [ ] 3. Run semantic indexes on the ChangeGuard project to visually audit PowerShell/CMD layout rendering.

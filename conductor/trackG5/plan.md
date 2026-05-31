## Plan: Track G5 Visual Intelligence & Navigation

### Phase 1: Native Visualization Export
- [ ] **Task 1.1**: Create `src/commands/viz.rs` implementing the `viz` command.
- [ ] **Task 1.2**: Implement `export_to_html(db: &CozoStorage)` to fetch all nodes/edges from CozoDB and generate a D3.js or Vis.js interactive file.
- [ ] **Task 1.3**: Add "Heat Map" support: nodes with high `risk_score` (from `HotspotProvider`) are rendered larger and in red.

### Phase 2: Community & Search Integration
- [ ] **Task 2.1**: Update the visualization template to support filtering by "Architectural Domain" (Community Label).
- [ ] **Task 2.2**: Implement a "Search to Center" feature in the HTML template to quickly find specific symbols.
- [ ] **Task 2.3**: Wire the `viz` command into `src/main.rs`.

### Phase 3: Verification (TDD)
- [ ] **Task 3.1**: Write a test verifying that `changeguard viz` produces a valid HTML file containing the expected number of node entries.
- [ ] **Task 3.2**: Verify that the generated HTML correctly embeds the community labels for each node.
- [ ] **Task 3.3**: Verify that the "Heat Map" correctly reflects the risk scores stored in CozoDB.

### Definition of Done (DoD)
- [x] ChangeGuard can generate an interactive architectural map without external Python scripts.
- [x] The map highlights hotspots and community boundaries.
- [x] No more than 4 files modified: `src/commands/viz.rs`, `src/commands/mod.rs`, `src/main.rs`, `src/index/graph_loader.rs`.

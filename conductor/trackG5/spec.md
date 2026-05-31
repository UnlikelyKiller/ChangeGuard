# Specification: Track G5 Visual Intelligence & Navigation

## Goal
Implement a native Rust command for exporting interactive architectural visualizations directly from the ChangeGuard Knowledge Graph.

## Context
Visualizing the graph is essential for understanding complex refactors. Currently, we rely on external Python scripts to generate `graph.html`. This track moves that logic into the Rust core, allowing users to run `changeguard viz` at any time.

## Technical Details

### 1. `viz` Command
Implement the `viz` command in `src/commands/viz.rs`.
- Query CozoDB for all `node` and `edge` entries.
- Export to a standalone HTML file with an embedded JS library (e.g., **Vis.js** or **D3.js**).

### 2. Highlighting & "Heat Maps"
- **Hotspots**: Integrate with the `HotspotProvider`. Nodes with high churn should be colored differently (e.g., red/large).
- **Domains**: Use the `community` labels to color-code clusters, making it easy to see where one architectural domain ends and another begins.

### 3. Interactive Templates
Embed a minified HTML template in the Rust binary. The template should support:
- Clicking a node to see its metadata.
- Searching for a symbol name to center the view.
- Toggling between "Structural" (code-only) and "Semantic" (full-graph) views.

## TDD Requirements
1.  **HTML Generation**: Verify that the command creates an HTML file with non-zero content.
2.  **Metadata Embedding**: Test that symbol metadata (e.g., file path, risk score) is correctly serialized into the HTML's JSON payload.
3.  **Command Wiring**: Verify that `changeguard viz` is accessible from the CLI.

## Definition of Done
- [ ] `changeguard viz` command implemented.
- [ ] Interactive HTML template created and embedded.
- [ ] Heat-map and community coloring support verified.
- [ ] No more than 4 files modified: `src/commands/viz.rs`, `src/commands/mod.rs`, `src/main.rs`, `src/index/graph_loader.rs`.

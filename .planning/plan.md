## Plan: Entity Graph Schema and Cross-Surface Links (Track W1)

### Phase 1: Define Enums and Struct Updates
- [ ] Task 1.1: Create `src/state/graph_kinds.rs` defining `NodeKind` and `EdgeKind` enums with `serde` support.
- [ ] Task 1.2: Update `GraphNode` and `GraphEdge` in `src/state/storage_cozo.rs` to use `NodeKind` and `EdgeKind` instead of `String`.
- [ ] Task 1.3: Define a metadata schema versioning standard (e.g. `schema_version: "v1"`) for graph node JSON properties.

### Phase 2: Stable ID Generation
- [ ] Task 2.1: Implement URN-based stable ID builder (`urn:changeguard:<kind>:<normalized_id>`) in a utility module (e.g., `src/platform/urn.rs`).
- [ ] Task 2.2: Ensure all file paths passed to the ID builder use forward slashes `/` via `camino::Utf8Path`.

### Phase 3: Traversal API
- [ ] Task 3.1: Implement `GraphTraversal` struct with `get_related_entities(seed_id, relation_kinds, max_hops)` query helper in `src/state/cozo/queries.rs` or a new module `src/state/graph_traversal.rs`.
- [ ] Task 3.2: Write corresponding Datalog templates to support arbitrary relation filters and max hop depth limits.

### Phase 4: Migration and Refactoring
- [ ] Task 4.1: Refactor `src/index/incremental.rs` to use the new enums and URN generator for `File`, `Symbol`, and `Calls` relationships.
- [ ] Task 4.2: Refactor `src/impact/enrichment/kg_provider.rs` to parse typed relations and support URN formatting.
- [ ] Task 4.3: Add a migration script to `src/state/cozo/init.rs` (`migrate_cozo_schema`) to translate existing nodes/edges to the new URN formats and strictly-typed enums.

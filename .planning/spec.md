# Track W1: Entity Graph Schema and Cross-Surface Links

## 1. Node and Edge Kinds (Rust Enums)
To transition from a stringly-typed graph to a strictly-typed graph, we will introduce `NodeKind` and `EdgeKind` enums in a new module (e.g., `src/state/graph/kinds.rs`).

### `NodeKind` Enum
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    File, // Retained for backwards compatibility with existing index
    Symbol,
    Endpoint,
    Service,
    DataModel,
    Migration,
    ConfigKey,
    DeploySurface,
    CiJob,
    Dependency,
    Test,
    ObservabilitySignal,
    Adr,
    LedgerTransaction,
    Hotspot,
    TemporalCoupling,
    SecurityBoundary,
}
```

### `EdgeKind` Enum
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Owns,
    Handles,
    Calls,
    Covers,
    Governs,
    Supersedes,
    Deploys,
    DependsOn,
    Emits,
    AlertsOn,
    ChangedWith,
    Validates,
    Authenticates,
    Authorizes,
    TouchesSecret,
}
```

## 2. CozoDB Schema Strategy
We will **keep the generic `node` and `edge` tables** rather than creating specialized tables for every entity type. 
- **Why:** Generic tables allow for uniform graph traversal (e.g., `*edge{source, target, relation}`) and straightforward full-text search (`node:fts_idx`). Splitting into specialized tables makes recursive graph path queries exponentially more complex in Datalog.
- **Enforcement:** Validation of `NodeKind` and `EdgeKind` will occur at the Rust boundary. The `category` and `relation` fields in `GraphNode` and `GraphEdge` structs will be updated to use the enums instead of raw strings.

### Schema Versioning in JSON Metadata
To support deterministic schema-versioned graph relations, `metadata` in `node` will require a standard payload shape. We will enforce JSON schemas containing a version field for any structured metadata.
```json
{
  "schema_version": "v1",
  "kind_specific_data": { ... }
}
```

## 3. Stable IDs Strategy
Nodes must have fully stable IDs that withstand cross-platform execution (Windows vs Linux).
We will use URN-style identifiers: `urn:changeguard:<kind>:<normalized_path_or_name>`

- **Path Normalization:** We will strictly enforce `camino::Utf8Path` and ensure all paths use forward slashes `/`. We will preserve the native case mapping from Git to ensure predictability. 
- **Examples:**
  - File: `urn:changeguard:file:src/index/incremental.rs`
  - Symbol: `urn:changeguard:symbol:src/index/incremental.rs:MyStruct::my_method`
  - Service: `urn:changeguard:service:auth_service`

## 4. Traversal API Design
We will introduce a specialized Graph API to query relations.

```rust
pub struct GraphTraversal<'a> {
    storage: &'a CozoStorage,
}

impl<'a> GraphTraversal<'a> {
    /// Recursively fetches related entities up to `max_hops`.
    /// Optionally filters by specific `relation_kinds`.
    pub fn get_related_entities(
        &self,
        seed_id: &str,
        relation_kinds: Option<Vec<EdgeKind>>,
        max_hops: usize
    ) -> Result<Vec<GraphNode>> {
        // Generates dynamic datalog limiting hops and relation types
        // E.g. ?[target] := *edge{source: $seed, target, relation}, is_in(relation, $rels)
        // Returns the list of resolved target GraphNodes.
        todo!()
    }
}
```

## 5. Migration Plan
1. **Add Enums:** Create `src/state/graph_kinds.rs` (or `kinds.rs` under an appropriate module) with `NodeKind` and `EdgeKind`.
2. **Update Rust Structs:** Modify `GraphNode` and `GraphEdge` in `src/state/storage_cozo.rs` to use the enums instead of raw strings.
3. **Refactor Indexers:** Update `src/index/incremental.rs` to use `NodeKind::File`, `NodeKind::Symbol`, and `EdgeKind::Calls` alongside the URN stable ID generation.
4. **Refactor KG Provider:** Update `src/impact/enrichment/kg_provider.rs` to query by URNs and specific kinds.
5. **Data Migration:** Update `init::migrate_cozo_schema` to rewrite old string-based categories and relations (`calls` -> `calls`, paths -> URNs) if upgrading from an older DB version.

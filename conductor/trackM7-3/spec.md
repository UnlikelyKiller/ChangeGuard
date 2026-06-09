# Specification: Track M7-3 — Data-Flow Coupling Risk

## Objective
Flag call chains where route handlers and their data models co-change, detect incomplete refactors that touch only part of a data-flow path.

## Components

### 1. Call Chain Enumeration (`src/index/call_graph.rs` extend)

```rust
pub fn enumerate_call_chains(
    call_graph: &CallGraph,
    routes: &[Route],
    max_depth: usize,
) -> Vec<CallChain>
```

Walk from each route handler through call-graph edges to a configurable max depth. Detect cycles and terminate at `max_depth`.

### 2. Data-Flow Coupling Detection (`src/coverage/dataflow.rs`)

```rust
pub fn compute_data_flow_coupling(
    call_chains: &[CallChain],
    changed_files: &[ChangedFile],
    data_models: &[DataModel],
    min_change_pct: f64,
) -> Vec<DataFlowMatch>
```

For each call chain:
1. Identify nodes that are in the changed file set
2. Identify nodes that are data model files
3. If ≥20% of chain nodes changed AND at least one node is a data model: flag as coupling
4. Skip chains shorter than 2 nodes
5. Filter out standard-library / framework nodes

### 3. Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChain {
    pub nodes: Vec<CallChainNode>,
    pub has_cycle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallChainNode {
    pub symbol: String,
    pub file_path: PathBuf,
    pub is_data_model: bool,
    pub is_external: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataFlowMatch {
    pub chain_label: String,
    pub changed_nodes: Vec<String>,
    pub total_nodes: usize,
    pub change_pct: f64,
    pub risk: RiskLevel,
}
```

### 4. Risk Impact

- **Route + data model co-changed**: Medium elevation. "Data-flow coupling: {route} and {model} changed together"
- **Call chain with 3+ changed nodes**: Medium per node. "Call chain affected: {chain}"
- **Chain depth > 5**: High elevation regardless of changed count

### 5. Impact Enrichment

`data_flow_matches: Vec<DataFlowMatch>` on `ImpactPacket`. Sorted by change_pct descending in `finalize()`, cleared in `truncate_for_context()`. Risk weight: 4 per match, cap at 20.

## Test Specifications

| Test | Assertion |
|---|---|
| Route→handler→model all changed | `DataFlowMatch` returned with correct chain |
| Only route changed, model not changed | No match (below 20% threshold) |
| Cycle in call graph | Chain terminates at max depth, `has_cycle = true` |
| Standard library node filtered | `println!` call does not create chain node |
| Chain shorter than 2 nodes | Skipped entirely |
| SQL table name as model fallback | `db.query("users")` → model "users" |
| Change percentage computed correctly | 3/10 changed = 30% |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **No hot-path embedding**: All data from pre-indexed call graph and data models.
- **Cycle-safe**: `max_depth` enforced strictly; cycles detected, not infinite-looped.
- **Config-driven**: `[coverage.data_flow].enabled = false` → no coupling detection.
- **Determinism**: `DataFlowMatch` implements `Ord` by `change_pct` descending.

## Hardening Additions (in plan)

| Addition | Reason |
|---|---|
| Cycle detection with depth limit | Prevents infinite loops on cyclic call graphs |
| 20% change-percentage threshold | Single-node changes in large chains are not coupling |
| External/stdlib node filtering | `std::fs::read` should not create data-flow chains |
| SQL table-name model resolution | Not all handlers reference explicit struct types |
| Minimum chain depth of 2 | Handler-only chains are trivia, not coupling |

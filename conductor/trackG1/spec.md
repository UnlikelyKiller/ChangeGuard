# Specification: Track G1 CozoDB Integration & Schema

## Goal
Implement the foundation of the ChangeGuard Knowledge Graph by integrating **CozoDB** as the primary storage engine for architectural relationships. This track moves ChangeGuard from a flat relational model (SQLite) to a hybrid Relational-Graph-Vector model.

## Context
`graphifyy` currently provides the graph data, but ChangeGuard needs a native way to query this data using **Datalog** for complex impact analysis (e.g., recursive reachability). CozoDB allows us to embed a high-performance graph engine directly in the Rust binary using SQLite as the persistent backend.

## Technical Details

### 1. Dependency
Add the `cozo` crate with the `storage-sqlite` feature. This allows ChangeGuard to maintain its single-file data storage while gaining graph capabilities.

### 2. Storage Wrapper (`src/state/storage/cozo.rs`)
Implement a `CozoStorage` struct that abstracts the `DbInstance`.
- `new(path: &Path)`: Initializes the database.
- `run_script(script: &str)`: Executes a Datalog script and returns results as `NamedRows`.
- `setup_schema()`: Ensures core relations exist on startup.

### 3. Datalog Schema
The graph will be stored in three core relations:

#### A. Node Relation
```datalog
:create node {
    id: String
    =>
    label: String,
    category: String,      # 'code', 'doc', 'rationale', 'domain'
    risk_score: Float,     # Seeded from hotspots, diffused via edges
    metadata: Json         # Extra attributes (line numbers, authors, etc.)
}
```

#### B. Edge Relation
```datalog
:create edge {
    source: String,
    target: String,
    relation: String
    =>
    confidence: Float,     # 0.0 to 1.0 (EXTRACTED vs INFERRED)
    provenance_id: String  # Link to ChangeGuard Ledger transaction
}
```

#### C. Ledger Link Relation
```datalog
:create ledger_link {
    node_id: String,
    ledger_id: String
    =>
    interaction_type: String # 'created', 'modified', 'referenced'
}
```

## TDD Requirements
1.  **Schema Init**: Test that calling `setup_schema()` multiple times is idempotent and correctly creates the relations.
2.  **Basic Persistence**: Insert a node and edge, then query them back using Datalog.
3.  **Recursive Reachability**: Verify that a query like `?[target] := *edge{source: 'A', target: target}` returns children, and `?[target] := *edge{source: 'A', target: t}, *edge{source: t, target: target}` returns grandchildren.

## Definition of Done
- [ ] `cozo` crate added to `Cargo.toml`.
- [ ] `CozoStorage` implemented with persistent SQLite backend support.
- [ ] Datalog schema defined and verified via unit tests.
- [ ] Reachability test passes with sample graph data.
- [ ] No more than 4 files modified.

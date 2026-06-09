# Track X4: `ledger graph` Writes Transaction→Entity Edges on Commit

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** High

## Objective

`changeguard ledger graph <tx-id>` returns an empty table for every transaction because no `edge{source: tx_urn, target: entity_id}` rows are ever written to CozoDB during `ledger commit`. Track W12 specified this link but did not implement the CozoDB edge-writing path.

## Problem Statement

`ledger_graph.rs` queries:
```datalog
?[entity_id, label, category, relation] :=
  *node{id: entity_id, label: label, category: category},
  *edge{source: $tx_urn, target: entity_id, relation: relation}
```

For this to return rows, two things must be true:
1. A `node` with `id = entity_id` exists (file/symbol nodes do exist from indexing).
2. An `edge` with `source = "urn:changeguard:transaction:{tx_id}"` exists pointing to those nodes.

The `LedgerTransaction` node itself is inserted during commit (`NodeKind::LedgerTransaction`), but no outgoing `Affects` edges are written from it to the file/symbol nodes it touched. As a result, the graph neighborhood is always empty.

## Acceptance Criteria

1. After `ledger commit <tx-id> --summary "..." --reason "..."`, CozoDB contains:
   - One `LedgerTransaction` node: `urn:changeguard:transaction:{tx_id}`.
   - One `Affects` edge per changed file: `edge{source: tx_urn, target: file_urn, relation: "affects"}`.
2. `ledger graph <tx-id>` shows one row per file/entity the transaction touched.
3. File URNs use the existing file node URN format from the KG (e.g., `urn:changeguard:file:src/commands/hotspots.rs` or whatever format `graph_loader.rs` uses for file nodes).
4. If a file is not yet in the KG (not indexed), the edge is still written; the query result will just not have a matching `node` row (acceptable — node must exist first).
5. Edge writing is transactional: if CozoDB is unavailable, a `warn!` is emitted but the SQLite ledger commit still succeeds.

## API Contracts

CozoDB edge written on commit:
```
edge{
  source: "urn:changeguard:transaction:{tx_id}",
  target: "urn:changeguard:file:{relative_path}",
  relation: "affects"
}
```

`ledger graph <tx-id>` output (non-empty):
```
Graph neighborhood for transaction: abc123...
Entity ID                                  Label              Category   Relation
urn:changeguard:file:src/commands/ask.rs   ask.rs             file       affects
...
```

## Key Files

- `src/ledger/transaction.rs` — `TransactionManager::commit_transaction` (edge writing site)
- `src/commands/ledger.rs` — `execute_ledger_commit` (orchestrates commit)
- `src/state/storage_cozo.rs` — `CozoStorage::insert_edges`
- `src/commands/ledger_graph.rs` — query (no change needed if schema matches)

## Definition of Done

- `changeguard ledger graph <tx-id>` shows ≥ 1 row for a recently committed transaction that touched at least one file.
- CozoDB edge writes are gated on `storage.cozo.is_some()`.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

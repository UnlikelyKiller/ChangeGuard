# Track Z6: Ledger Graph Transaction Edges
 
**Status:** In Progress
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** High

## Objective

Populate outgoing `Affects` edges from `LedgerTransaction` nodes in CozoDB when a transaction commits, so that `ledger graph <tx-id>` is not empty.

## Problem Statement

When a user commits a transaction using `ledger commit`, `execute_ledger_commit()` saves the transaction metadata and inserts a `NodeKind::LedgerTransaction` node in CozoDB. However, it never inserts the outgoing `Affects` edges from the transaction node to the nodes of the files modified in the transaction. As a result, the transaction graph neighborhood remains empty.

## Acceptance Criteria

1. On transaction commit (`ledger commit`), `TransactionManager::commit_transaction` queries the transaction's changed files from the ledger state database.
2. For each changed file, it inserts a `GraphEdge` in CozoDB with `source = urn:changeguard:transaction:{tx_id}`, `target = urn:changeguard:file:{relative_path}`, and `relation = EdgeKind::Affects` (Datalog: `'affects'`).
3. Running `changeguard ledger graph <tx-id>` lists the affected file nodes correctly.

## Key Files

* `src/ledger/transaction.rs` — `TransactionManager::commit_transaction()`
* `src/commands/ledger_graph.rs` — `execute_ledger_graph()`

## Definition of Done

* `cargo nextest run --lib --bins --workspace` passes.
* Verifying `ledger graph <tx-id>` returns files modified during a committed transaction.

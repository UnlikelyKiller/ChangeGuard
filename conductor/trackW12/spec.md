# Track W12 Spec: Ledger Transaction Entity Links and Validator UX

## Background

Provenance and ledger transaction tracking currently scores 9/10. ChangeGuard already has lifecycle, drift detection, signing, federation, ADR generation, and commit validators. The remaining gap is validator lifecycle UX and richer links from transactions to graph entities.

## Objective

Raise ledger provenance tracking to 10/10 by adding validator IDs and lifecycle commands, transaction graph links, hook diagnostics, and stable provenance export.

## Proposed Design

1. Add validator IDs and `ledger validator list`, `disable`, `enable`, `remove`, and `doctor`.
2. Add entity-link tables from ledger transactions to symbols, endpoints, services, data models, config keys, tests, ADRs, deploy surfaces, dependencies, observability signals, hotspots, and security boundaries.
3. Add hook lifecycle diagnostics and repair commands for sidecar/pending mismatches.
4. Add `ledger graph <tx-id>` to show the entity neighborhood governed by a transaction.
5. Add versioned provenance export for audit and external ingestion.

## Critical Files

| File | Expected work |
|---|---|
| `src/ledger/transaction.rs` | Link transactions to graph entities |
| `src/ledger/db.rs` | Add validator IDs and link persistence |
| `src/commands/ledger.rs` | Add validator lifecycle and graph commands |
| `src/commands/ledger_stack.rs` | Reuse or migrate stack display to validator UX |
| `src/hooks/` or hook templates | Add hook diagnostics and repair guidance |

## Definition of Done

- Validators can be listed, disabled, enabled, removed, diagnosed, and referenced by stable ID.
- Ledger transactions can explain their linked graph neighborhood.
- Hook-created pending transaction edge cases have an auditable repair path.
- Target score after completion: 10/10.

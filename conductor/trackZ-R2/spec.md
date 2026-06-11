# Track Z-R2: Ledger Adopt Path Deduplication & Defense-in-Depth

**Status:** Planned
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** High

## Objective

Eliminate redundant Knowledge Graph writes in `execute_ledger_adopt`, centralize synthetic-entity filtering, and harden `get_transaction_files` against non-path transaction entities.

## Problem Statement

The `ledger adopt` command performs two overlapping KG writes:

1. `tx_mgr.commit_change(...)` writes a `LedgerTransaction` node and `Affects` edges.
2. After dropping the manager, `write_ledger_graph_edges(...)` writes the **same** transaction node and edges again.

This is architectural drift: the adopt path bypasses the internal `commit_change` file-discovery logic but still triggers it, then applies its own supplemental logic. If `insert_nodes` ever stops being upsert-safe, the second call will fail.

Additionally, `write_ledger_graph_edges` does not filter synthetic entities (the `drift_adoption:` / UUID check exists only in `commit_change`), creating a defense-in-depth gap. And `get_transaction_files` unconditionally inserts `tx.entity_normalized` into the file set, which for a single-item adopt is the adopted drift transaction's UUID—a non-file path that leaks into the URN builder unless caught downstream.

## Acceptance Criteria

1. **Single KG write path**: `execute_ledger_adopt` must produce exactly one `LedgerTransaction` node and one `Affects` edge per real changed file in CozoDB.
2. **Backward-compatible API**: Existing `commit_change` callers must compile unchanged.
3. **Synthetic filtering in `write_ledger_graph_edges`**: Any file string containing `drift_adoption:` or parseable as a UUID is skipped.
4. **`get_transaction_files` hardening**: `entity_normalized` is only inserted into the file set if it looks like a file path (contains `/` or `.`) and is not synthetic.
5. **No behavior change** for normal `ledger commit` or `ledger atomic` paths.

## Key Files

- `src/ledger/transaction.rs` — `commit_change` and `get_transaction_files`.
- `src/commands/ledger/maintenance.rs` — `execute_ledger_adopt` and `write_ledger_graph_edges`.
- `tests/integration/ledger_graph_edges.rs` — Existing adopt test.

## Definition of Done

- `cargo nextest run --lib --bins --workspace` passes.
- `cargo nextest run --test integration` passes (including `test_adopt_writes_kg_edges_with_real_files`).
- `ledger adopt --all` produces exactly one `LedgerTransaction` node and one `Affects` edge per real file in CozoDB.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean.

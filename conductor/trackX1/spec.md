# Track X1: `ask` Falls Back to KG Context When Semantic Index Is Absent

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** High

## Objective

`changeguard ask` in global mode currently requires a populated semantic vector store (Tantivy embeddings). When the semantic index is empty or stale — but the CozoDB KG has thousands of nodes — the command either returns generic answers or errors with "semantic index is empty". The KG is rich enough to provide relevant context through BM25 text search plus graph traversal, so `ask` should use it when embeddings are unavailable.

## Problem Statement

Running `changeguard ask "what are the hotspot commands?"` when the Tantivy semantic index is empty queries the vector store, finds nothing, and passes an empty context window to the LLM. The KG at that moment has 7,324+ nodes and 30,000+ edges, but `ask` never queries them. Result: the LLM answers from general training knowledge rather than project-specific facts.

## Acceptance Criteria

1. When `ask` is invoked in global mode and the semantic vector store is empty or returns 0 results for the query, it falls back to CozoDB BM25 node-label search with the same query terms.
2. The fallback is transparent: a single `note` line is printed before the answer (`Note: semantic index empty — using KG text search for context`).
3. When both semantic and KG results are absent, the LLM is still called with an explicit `no project context available` note in the system prompt.
4. Fallback logic is gated: if the user passes `--no-kg-fallback`, KG fallback is skipped.
5. All existing `ask` tests pass; one new integration test covers the KG-only path.

## API Contracts

```
changeguard ask [--no-kg-fallback] [--global] "query"
```

No new CLI flags required beyond `--no-kg-fallback` (optional).

KG fallback query (CozoDB):
```datalog
?[id, label, category] := *node{id, label, category},
  label ~ $query_terms
  :limit 20
```

## Key Files

- `src/commands/ask.rs` — primary entry point for `execute_ask`
- `src/retrieval/` — semantic retrieval pipeline
- `src/state/storage_cozo.rs` — `CozoStorage::run_script_with_params`
- `src/bridge/client.rs` — LLM call site

## Definition of Done

- `changeguard ask "query"` returns a project-grounded answer using KG when no semantic index exists.
- The fallback note appears only when KG path is taken.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

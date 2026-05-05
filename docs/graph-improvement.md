# Research: Knowledge Graph Integration for ChangeGuard

This document outlines the proposed implementation, integration, and architectural decisions for incorporating Knowledge Graph (KG) features—inspired by `graphifyy`—directly into ChangeGuard.

## 1. Objectives

*   **Move beyond file-level impact**: Transition from simple file-change detection to "Conceptual Blast Radius" analysis.
*   **Unify Code & Knowledge**: Explicitly link source code abstractions (classes, functions) to design rationales, documentation, and external research papers.
*   **Audit-Aware Intelligence**: Link graph relationships to the ChangeGuard ledger to understand how relationships evolved over time.
*   **Semantic Risk Scoring**: Use graph topology metrics (centrality, community density) to refine risk assessments.
*   **Local-First Intelligence**: Ensure high-fidelity architectural reasoning can run entirely on consumer hardware (e.g., 12GB VRAM GPUs).

---

## 2. Recommended Database: CozoDB

After evaluating several Rust-compatible options (SurrealDB, OverGraph, IndraDB), **CozoDB** is the recommended choice for ChangeGuard.

### Why CozoDB?
1.  **Datalog Query Engine**: Datalog is a declarative logic language superior to SQL for complex, recursive graph queries (e.g., "Find all design rationales within 3 hops of this changed function").
2.  **Relational-Graph-Vector Hybrid**: CozoDB unifies the three pillars ChangeGuard needs:
    *   **Relational**: For the ledger, snapshots, and structured metadata.
    *   **Graph**: For the architectural relationships and dependency maps.
    *   **Vector**: For semantic similarity search and RAG (Knowledge Enrichment).
3.  **Time Travel (MVCC)**: CozoDB supports temporal queries, allowing ChangeGuard to query the state of the system graph at any point in the ledger history.
4.  **Lightweight & Embedded**: It is a pure Rust library that can use **SQLite** as its storage engine, maintaining ChangeGuard's local-first, zero-infrastructure philosophy.

---

## 3. Implementation Architecture

### A. Data Schema (Datalog)
We will define three core relations in CozoDB:

```datalog
# Nodes represent any entity (File, Function, Rationale, Paper, Community)
:create node {
    id: String,
    =>
    label: String,
    category: String,      # 'code', 'doc', 'paper', 'rationale', 'domain'
    risk_score: Float,     # Diffusion score from hotspots
    metadata: Json
}

# Edges represent relationships
:create edge {
    source: String,
    target: String,
    relation: String,      # 'calls', 'rationale_for', 'cites', 'semantically_similar', 'belongs_to'
    =>
    confidence: Float,
    provenance_id: String  # Link to ChangeGuard Ledger ID
}

# Ledger Integration
:create ledger_link {
    node_id: String,
    ledger_id: String
    =>
    interaction_type: String # 'created', 'modified', 'referenced'
}
```

### B. The Integration Pipeline
ChangeGuard will orchestrate the extraction pipeline while maintaining the "Source of Truth" in its own CozoDB instance.

1.  **Extraction (Hardened)**: ChangeGuard triggers the local LLM extraction pipeline.
    *   **Token Budgeting**: Enforces a strict **30,000 token budget** to fit within local 38k KV caches (optimized for Qwen 3.5 9B on 12GB VRAM).
    *   **Adaptive Recursion**: Automatically splits dense document chunks (e.g., in `docs/` or `conductor/`) when responses are truncated.
2.  **Semantic Labeling**: Uses the local LLM to translate raw Leiden clusters into **Architectural Domains**. This converts "Community 14" into "Analysis Status & Impact Packet Serialization," providing a human-centric navigation layer.
3.  **Ingestion**: A new Rust module `src/kg/ingest.rs` parses the labeled `graph.json` and bulk-loads it into CozoDB, correlating entities with the ChangeGuard `ledger`.
4.  **Risk Diffusion**: ChangeGuard runs a custom algorithm:
    *   Seed risk from `HotspotProvider` (high churn files).
    *   Propagate risk through the graph edges (`calls`, `shares_data_with`, `belongs_to`).
    *   Result: A global "Architectural Fragility Map."

---

## 4. Feature Integration: `KGEnrichmentProvider`

The current `ImpactOrchestrator` will gain a new `KGProvider`.

### Query Flow:
1.  **Input**: A set of changed files in an `ImpactPacket`.
2.  **Datalog Query**:
    ```datalog
    # Find all 'rationale' nodes within 2 hops of changed files
    ?[rationale_id, text, distance] := 
        *changed_files[file_id],
        *node{id: file_id},
        *edge_path[file_id, rationale_id, distance],
        *node{id: rationale_id, category: 'rationale', metadata: {content: text}},
        distance <= 2
    ```
3.  **Output**: The `ImpactPacket` is enriched with "Related Rationales," warning the developer if their code change contradicts a documented design principle.

---

## 5. Proposed Improvements over standard Graphify

1.  **Bi-Directional Provenance**: While basic graphs know *what* links to *what*, ChangeGuard will know *when* and *why* by linking graph edges to ledger transactions.
2.  **Contract Enforcement**: We can define "Negative Edges" (Forbidden Paths). ChangeGuard's `verify` command can then check if a change accidentally violates architectural boundaries (e.g., "Internal Ledger logic must not be called from the CLI layer").
3.  **Dynamic Blast Radius**: Instead of a static file list, the blast radius becomes a "Heat Map" on the knowledge graph, identifying "Community Fragility" based on real-time churn.

---

## 6. Development Roadmap

### Phase 1: Local Intelligence Hardening (Completed)
*   Verified local Qwen 3.5 inference on Intel Arc B580.
*   Implemented adaptive chunking and 30k token budgeting for large corpora.
*   Validated community labeling as a semantic bridge.

### Phase 2: Storage Migration
*   Add `cozodb` crate to ChangeGuard.
*   Port existing SQLite schema to CozoDB relations in `src/state/storage.rs`.
*   Implement `src/state/cozo.rs` wrapper for Datalog queries.

### Phase 3: Ingestion & Enrichment
*   Implement `graph.json` parser in Rust.
*   Build the mapping logic to correlate graph nodes with ledger entries.
*   Integrate `KGEnrichmentProvider` into the `ImpactOrchestrator`.

### Phase 4: Visual Intelligence
*   Export `graph.html` directly from CozoDB data.
*   Highlight "Impact Zones" in the visualization using real-time orchestration data.

### Phase 5: Native Extraction (The "De-coupling")
*   **Structural Port**: Migrate AST link discovery logic to ChangeGuard's native Rust `tree-sitter` implementation.
*   **Semantic Port**: Move chunking and LLM orchestration logic from Python to the `src/ai/` module in Rust.
*   **The "Cord-Cut"**: Remove the dependency on the external `graphifyy` package, making ChangeGuard a truly standalone, single-binary intelligence tool.

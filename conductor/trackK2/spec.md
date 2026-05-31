# Track K2: Intelligence Precision (Adaptive Inference Context)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`changeguard ask --semantic` hallucinations occur when no transaction is active. The system defaults to explaining the (empty) impact report instead of querying the indexed codebase. This leads to responses like "Based on the provided impact summary, there is no information about X."

## Solution
1. **Adaptive Inference Mode**: If no transaction is active (clean state), switch the system prompt from "Changes Analyst" to "Codebase Oracle".
2. **Context Pivot**: 
    - Skip `ImpactPacket` pruning and formatting if it contains zero changes.
    - Allocate 90% of the token budget to codebase chunks retrieved via RAG.
3. **Retrieval Hardening**: 
    - Implement a "Query Refiner" that extracts keywords from the user question to improve HNSW search relevance.
    - Set `top_k` dynamically based on available context window (fill the budget).
4. **Source Attribution**: Require the model to cite the specific file paths and line numbers provided in the retrieved chunks.

## Definition of Done (DoD)
- [ ] `changeguard ask --semantic "How does the search engine work?"` returns an architecture explanation derived from code chunks in clean git state.
- [ ] Response includes explicit citations (e.g. `[src/search/mod.rs]`).
- [ ] No mention of "impact summary" when git state is clean.
- [ ] Regression test: verify context assembly token distribution for clean vs dirty states.
- [ ] CI gate passes.

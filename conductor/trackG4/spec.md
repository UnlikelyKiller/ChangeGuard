# Specification: Track G4 Semantic Impact Orchestration

## Goal
Implement the core "Semantic Intelligence" feature: extending impact analysis to include conceptual neighbors and architectural domains discovered by the Knowledge Graph.

## Context
Standard impact analysis stops at code dependencies. This feature uses the graph to find "Conceptual Neighbors"—for example, realizing that changing the `Redaction` logic impacts the `Safety Policy` documentation, even if no code import exists between them.

## Technical Details

### 1. `KGEnrichmentProvider`
Implement a new provider in `src/impact/enrichment/kg_provider.rs`:
- Input: List of changed files.
- Logic: Performs Datalog queries on CozoDB.
- Output: List of semantically related symbols and documentation.

### 2. Datalog Impact Queries
Implement two core queries:
- **Neighbor Discovery**: `?[neighbor] := *edge{source: changed_id, target: neighbor, relation: 'semantically_similar'}` (and inverse).
- **Domain Context**: Find the community label (e.g., "Ledger Management") for each changed file to provide high-level context in the report.

### 3. Orchestration
Update `src/impact/orchestrator.rs` to include the `KGProvider` in the enrichment loop. The results should be stored in a new `semantic_impact` field in the `ImpactPacket`.

## TDD Requirements
1.  **Direct Neighbor Test**: Verify that if Node A and Node B are linked by a `semantically_similar` edge, changing A returns B as a neighbor.
2.  **Transitive Impact**: Test 2-hop reachability (e.g., A → B → C) where the link is a mix of `calls` and `rationale_for`.
3.  **Community Labeling**: Verify that a symbol's community name is correctly retrieved and added to the packet.

## Definition of Done
- [ ] `KGEnrichmentProvider` implemented and integrated.
- [ ] `ImpactPacket` extended with semantic data.
- [ ] Datalog queries verified via unit tests.
- [ ] No more than 4 files modified: `src/impact/enrichment/kg_provider.rs`, `src/impact/orchestrator.rs`, `src/impact/packet.rs`, `src/impact/enrichment/mod.rs`.

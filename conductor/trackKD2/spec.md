# Specification: Transitive Closure Reachability in KGProvider (Track KD2)

## Overview
Refactor ChangeGraph's impact analysis to utilize true recursive Datalog transitive closure. This replaces the hardcoded 1-hop and 2-hop reachability checks in `KGProvider` with a parameterized query that traces transitive dependencies to arbitrary depth.

## Architecture & SRP
- **Module**: `src/impact/enrichment/kg_provider.rs`
- **Responsibility**: Identify how codebase modifications transitively impact downstream modules/symbols.

## Requirements
- Replace the separate 1-hop and 2-hop query string formatters in `kg_provider.rs` with a single recursive Datalog rule query.
- Make the maximum recursion depth configurable via the `CoverageConfig` configuration model (default to a sensible depth like `4` or `5`).
- Ensure the query remains performant and utilizes Cozo's execution parallelism to prevent long execution times.
- Retain existing contract compatibility for output metrics (`KGImpact` elements, path lengths).

## Dependencies
- Track KD1 must be completed.

# Track K4: Automated Service-Map Extraction

## Status
Planning

## Milestone
K: Service Discovery & Storage Hardening

## Problem
In large poly-repo or multi-service monorepos, ChangeGuard lacks a formal "Service Map" that identifies logical service boundaries and cross-service dependencies. Impact analysis is currently file-based, which can obscure higher-level architectural risks (e.g. "Changing this data model breaks the downstream 'Billing' service").

## Solution
1. **Boundary Inference**: Use file path heuristics and `api_routes` topology to group files into logical services.
2. **Dependency Mapping**: Extract cross-service calls (via HTTP/gRPC patterns or Knowledge Graph reachability) to build a directed service graph.
3. **Service-Level Risk**: Assign risk scores to services based on churn, complexity, and incoming dependency volume.

## Definition of Done (DoD)
- [ ] `changeguard index` generates a `service_map.json` (or Cozo relation).
- [ ] `changeguard viz` includes a "Service View" mode.
- [ ] Impact analysis includes a "Cross-Service Impact" section.
- [ ] CI gate passes.

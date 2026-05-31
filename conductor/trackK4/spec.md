# Track K4: Service Boundary & Communication Mapping

## Status
Planning

## Milestone
K: Service Discovery & Storage Hardening

## Problem
Impact analysis is currently file-centric. In multi-service repositories, it fails to capture service-level blast radius (e.g. "Changing this data model affects the 'Ordering' service because it's a consumer of the 'Inventory' API").

## Solution
1. **Service Boundary Inference**:
    - **Marker-based**: Detect `Cargo.toml`, `package.json`, `go.mod`, `pom.xml`, and `Dockerfile` in subdirectories. Each directory containing these is a logical service root.
    - **Community-based**: Use the Leiden algorithm on the call graph to group orphan files into functional modules.
2. **Communication Extraction**:
    - **Pattern-based**: Detect HTTP client calls (`fetch`, `axios`, `ureq`) and server route definitions.
    - **Graph-based**: Link services where a caller in Service A targets a public API in Service B.
3. **Cross-Service Impact**:
    - Add `service_impact` to `ImpactPacket`.
    - Flag "Downstream Breakage" when a public contract in a service root is modified.

## Definition of Done (DoD)
- [ ] `changeguard index` identifies logical service boundaries and stores them in CozoDB.
- [ ] `changeguard viz` includes a "Service Connectivity Graph".
- [ ] `scan --impact` identifies cross-service dependencies in the human-readable report.
- [ ] Regression test: monorepo fixture with 2 services; verify call from A to B is captured.
- [ ] CI gate passes.

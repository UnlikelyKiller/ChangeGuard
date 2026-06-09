# Track W4 Plan: Service Boundary Ownership and Async Topology

- [x] Task W4.1: Create service overlay fixtures for monorepo roots, owners, queues, topics, data stores, and deploy manifests.
- [x] Task W4.2: Write tests for inferred plus declared service merge behavior and conflict handling.
- [x] Task W4.3: Add service overlay config parsing and validation.
- [x] Task W4.4: Add async topology extraction for queue/topic producer and consumer patterns.
- [x] Task W4.5: Emit W1 graph edges for service dependencies and ownership.
- [x] Task W4.6: Add service boundary impact rules and verification hints.
- [x] Task W4.7: Implement `changeguard services diff` with human and JSON output.
- [x] Task W4.8: Run service, impact, config, and full verification gates; reinstall.

## Definition of Done Checklist

- [x] Services have explicit owners and topology when metadata is present.
- [x] Missing metadata is surfaced as actionable unknowns.
- [x] Async boundary changes appear in impact output.
- [x] Full verification gate passes.

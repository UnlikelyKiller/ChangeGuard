# Track W4 Plan: Service Boundary Ownership and Async Topology

- [ ] Task W4.1: Create service overlay fixtures for monorepo roots, owners, queues, topics, data stores, and deploy manifests.
- [ ] Task W4.2: Write tests for inferred plus declared service merge behavior and conflict handling.
- [ ] Task W4.3: Add service overlay config parsing and validation.
- [ ] Task W4.4: Add async topology extraction for queue/topic producer and consumer patterns.
- [ ] Task W4.5: Emit W1 graph edges for service dependencies and ownership.
- [ ] Task W4.6: Add service boundary impact rules and verification hints.
- [ ] Task W4.7: Implement `changeguard services diff` with human and JSON output.
- [ ] Task W4.8: Run service, impact, config, and full verification gates; reinstall.

## Definition of Done Checklist

- [ ] Services have explicit owners and topology when metadata is present.
- [ ] Missing metadata is surfaced as actionable unknowns.
- [ ] Async boundary changes appear in impact output.
- [ ] Full verification gate passes.

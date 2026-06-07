# Track W2 Plan: Public API Endpoint Ownership, Auth, and Consumer Graph

- [ ] Task W2.1: Write endpoint graph fixture tests covering route-only, OpenAPI-only, and matched route/OpenAPI cases.
- [ ] Task W2.2: Add auth extraction tests for middleware and route-level evidence.
- [ ] Task W2.3: Add consumer extraction tests for HTTP callsites and generated client references.
- [ ] Task W2.4: Implement endpoint graph node and edge emission using W1 helpers.
- [ ] Task W2.5: Implement config overlay support for owner/auth/consumer facts that cannot be inferred.
- [ ] Task W2.6: Add endpoint impact rules for removed routes, schema changes, auth weakening, and known-consumer exposure.
- [ ] Task W2.7: Add `changeguard endpoints --json` and `changeguard endpoints --changed`.
- [ ] Task W2.8: Add docs and command examples.
- [ ] Task W2.9: Run focused endpoint/contract tests, then the full verification gate and reinstall.

## Definition of Done Checklist

- [ ] Route, contract, auth, owner, schema, and consumer links are visible in JSON.
- [ ] Unknown facts remain distinguishable from false facts.
- [ ] Endpoint-related impact output names the affected consumers and tests when known.
- [ ] Full verification gate passes.

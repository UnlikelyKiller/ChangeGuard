# Track W1 Plan: Entity Graph Schema and Cross-Surface Links

- [ ] Task W1.1: Inventory current node/link outputs from route, symbol, service, data model, env, deploy, dependency, test, observability, ledger, hotspot, and security code.
- [ ] Task W1.2: Write schema tests for stable node IDs, edge kinds, deterministic sorting, and backward-compatible JSON output.
- [ ] Task W1.3: Add CozoDB relations and any SQLite mirror migrations needed for entity links.
- [ ] Task W1.4: Implement common graph-link types and traversal helpers.
- [ ] Task W1.5: Refactor one existing provider to use the helper path as the integration proof.
- [ ] Task W1.6: Add bridge export coverage for the new relation schema.
- [ ] Task W1.7: Add migration regression tests against an existing ledger/index fixture.
- [ ] Task W1.8: Run `changeguard index --analyze-graph` and confirm graph output is deterministic across two runs.
- [ ] Task W1.9: Run the full verification gate and reinstall the binary.
- [ ] Task W1.10: Update `docs/TrackingAbility.md` if the foundation changes target scores or command names.

## Definition of Done Checklist

- [ ] Typed graph relation schema exists and is documented in code comments or command help.
- [ ] All new output is versioned and deterministic.
- [ ] At least one impact provider consumes the shared traversal helper.
- [ ] Full verification gate passes.
- [ ] Ledger transaction is committed with links to the source commit.

# Track W1 Plan: Entity Graph Schema and Cross-Surface Links

- [x] Task W1.1: Inventory current node/link outputs from route, symbol, service, data model, env, deploy, dependency, test, observability, ledger, hotspot, and security code.
- [x] Task W1.2: Write schema tests for stable node IDs, edge kinds, deterministic sorting, and backward-compatible JSON output.
- [x] Task W1.3: Add CozoDB relations and any SQLite mirror migrations needed for entity links.
- [x] Task W1.4: Implement common graph-link types and traversal helpers.
- [x] Task W1.5: Refactor one existing provider to use the helper path as the integration proof.
- [x] Task W1.6: Add bridge export coverage for the new relation schema.
- [x] Task W1.7: Add migration regression tests against an existing ledger/index fixture.
- [x] Task W1.8: Run `changeguard index --analyze-graph` and confirm graph output is deterministic across two runs.
- [x] Task W1.9: Run the full verification gate and reinstall the binary.
- [x] Task W1.10: Update `docs/TrackingAbility.md` if the foundation changes target scores or command names.

## Definition of Done Checklist

- [x] Typed graph relation schema exists and is documented in code comments or command help.
- [x] All new output is versioned and deterministic.
- [x] At least one impact provider consumes the shared traversal helper.
- [x] Full verification gate passes.
- [x] Ledger transaction is committed with links to the source commit.

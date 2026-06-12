## Plan: Track KD2 - Transitive Closure Reachability in KGProvider

### Phase 1: Configuration & Recursive Datalog Design
- [ ] Task 1.1: Add `max_reachability_depth` to `CoverageConfig` in `src/config/model.rs` (default to `5`).
- [ ] Task 1.2: Design and benchmark a recursive Datalog rule in CozoDB to compute transitive closure from seed files up to `max_depth` with cycle-detection.
  ```datalog
  seeds[id] <- $seed_list
  reachable[t, r, len] := seeds[s], *edge{source: s, target: t, relation: r}, len = 1
  reachable[t, r, len] := reachable[m, r, len_prev], *edge{source: m, target: t, relation: r}, len = len_prev + 1, len <= $max_depth
  ?[t, r, len] := reachable[t, r, len]
  ```

### Phase 2: Implementation & Refactoring
- [ ] Task 2.1: Refactor `KGProvider::enrich()` in `src/impact/enrichment/kg_provider.rs` to substitute the separate 1/2-hop queries with the single recursive query using parameterized arguments.
- [ ] Task 2.2: Ensure risk score propagation uses the new Datalog reachability logic where relevant.

### Phase 3: Verification
- [ ] Task 3.1: Run `cargo test` and ensure existing `test_kg_enrichment` pass.
- [ ] Task 3.2: Write a new test with 4-hop and 5-hop dependencies to verify arbitrary-depth reachability behaves correctly.

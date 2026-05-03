## Plan: Track M7-3 — Data-Flow Coupling Risk

### Phase 1: Call Chain Enumeration
- [x] Task 1.1: Extend `src/index/call_graph.rs` with `enumerate_call_chains(max_depth)`.
- [x] Task 1.2: Implement cycle detection (visited-set tracking).
- [x] Task 1.3: Filter external/stdlib nodes from chains.
- [ ] Task 1.4: Write test: chain enumeration from route handler.
- [ ] Task 1.5: Write test: cycle terminates at max depth.
- [ ] Task 1.6: Write test: `println!`/`std::fs` calls excluded from chains.

### Phase 2: Data-Flow Coupling Detection
- [x] Task 2.1: Implement `compute_data_flow_coupling()` in `src/coverage/dataflow.rs`.
- [x] Task 2.2: Implement 20% change-percentage threshold.
- [ ] Task 2.3: Implement SQL table-name model resolution fallback.
- [x] Task 2.4: Enforce minimum chain depth of 2.
- [x] Task 2.5: Write test: route+handler+model all changed → DataFlowMatch returned.
- [x] Task 2.6: Write test: only route changed → no match (below 20%).
- [ ] Task 2.7: Write test: 3/10 nodes changed → 30% → match.
- [ ] Task 2.8: Write test: chain depth 1 (handler only) → skipped.

### Phase 3: Types
- [x] Task 3.1: Define `CallChain`, `CallChainNode`, `DataFlowMatch` types.
- [x] Task 3.2: Implement `Ord` for `DataFlowMatch` (by `change_pct` descending).
- [x] Task 3.3: Add `data_flow_matches: Vec<DataFlowMatch>` to `ImpactPacket`.
- [x] Task 3.4: Wire `finalize()` sort and `truncate_for_context()` clear.
- [ ] Task 3.5: Write test: serialization roundtrip.
- [ ] Task 3.6: Write test: `finalize()` sorts by change_pct descending.
- [ ] Task 3.7: Write test: `truncate_for_context()` clears field.

### Phase 4: Risk Enrichment
- [x] Task 4.1: Wire `compute_data_flow_coupling` into `execute_impact()` enrichment.
- [/] Task 4.2: Implement risk weight: 4 per match, cap at 20. (Current implementation uses RiskElevation; need to align with numeric weight).
- [ ] Task 4.3: Write test: route+model co-change → risk reason added.
- [ ] Task 4.4: Write test: `[coverage.data_flow].enabled = false` → no enrichment.
- [ ] Task 4.5: Write test: weight capping (6 matches → weight 20, not 24).

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 5.2: Run `cargo test coverage::dataflow` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.

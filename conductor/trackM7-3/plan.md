## Plan: Track M7-3 — Data-Flow Coupling Risk

### Phase 1: Call Chain Enumeration
- [x] Task 1.1: Extend `src/index/call_graph.rs` with `enumerate_call_chains(max_depth)`.
- [x] Task 1.2: Implement cycle detection (visited-set tracking).
- [x] Task 1.3: Filter external/stdlib nodes from chains.
- [x] Task 1.4: Write test: chain enumeration from route handler.
- [x] Task 1.5: Write test: cycle terminates at max depth.
- [x] Task 1.6: Write test: `println!`/`std::fs` calls excluded from chains.

### Phase 2: Data-Flow Coupling Detection
- [x] Task 2.1: Implement `compute_data_flow_coupling()` in `src/coverage/dataflow.rs`.
- [x] Task 2.2: Implement 20% change-percentage threshold.
- [x] Task 2.3: Implement SQL table-name model resolution fallback.
- [x] Task 2.4: Enforce minimum chain depth of 2.
- [x] Task 2.5: Write test: route+handler+model all changed → DataFlowMatch returned.
- [x] Task 2.6: Write test: only route changed → no match (below 20%).
- [x] Task 2.7: Write test: 3/10 nodes changed → 30% → match.
- [x] Task 2.8: Write test: chain depth 1 (handler only) → skipped.

### Phase 3: Types
- [x] Task 3.1: Define `CallChain`, `CallChainNode`, `DataFlowMatch` types.
- [x] Task 3.2: Implement `Ord` for `DataFlowMatch` (by `change_pct` descending).
- [x] Task 3.3: Add `data_flow_matches: Vec<DataFlowMatch>` to `ImpactPacket`.
- [x] Task 3.4: Wire `finalize()` sort and `truncate_for_context()` clear.
- [x] Task 3.5: Write test: serialization roundtrip.
- [x] Task 3.6: Write test: `finalize()` sorts by change_pct descending.
- [x] Task 3.7: Write test: `truncate_for_context()` clears field.

### Phase 4: Risk Enrichment
- [x] Task 4.1: Wire `compute_data_flow_coupling` into `execute_impact()` enrichment.
- [x] Task 4.2: Implement risk weight: 4 per match, cap at 20. *(Implemented in `analyze_risk` with config-driven cap; default cap is 12.)*
- [x] Task 4.3: Write test: route+model co-change → risk reason added.
- [x] Task 4.4: Write test: `[coverage.data_flow].enabled = false` → no enrichment.
- [x] Task 4.5: Write test: weight capping (6 matches → cap enforced, not unbounded).

### Phase 5: Final Validation
- [x] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`. *(Full CI gate passes.)*
- [x] Task 5.2: Run `cargo test coverage::dataflow` — all tests pass. *(6/6 pass.)*
- [x] Task 5.3: Run full `cargo test` — no regressions. *(750 tests pass, 0 failures.)*

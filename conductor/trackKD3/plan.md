## Plan: Track KD3 - Declarative Logical & Security Checks in Datalog

### Phase 1: Declarative Datalog Security Rules
- [x] Task 1.1: Design a Datalog rule to identify security policy boundary violations (e.g. policies not covering all endpoints/resources they are bound to).
- [x] Task 1.2: Port authorization matching from `security.rs` loops to a Datalog query.
- [x] Task 1.3: Update `execute_boundaries` to query the Datalog rule results directly and output formatted violations.

### Phase 2: Reachability Rules Consolidation
- [x] Task 2.1: Consolidate `dead_code/evidence.rs` Datalog reachability queries into standard parameterized rules.
- [x] Task 2.2: Implement sink-to-sink reachability checks and data-flow violations in CozoDB.

### Phase 3: Verification
- [x] Task 3.1: Execute `cargo test --workspace` to ensure all existing security and dead-code checks function correctly.
- [x] Task 3.2: Write tests validating that the declarative rules yield identical results to the legacy imperative loops.

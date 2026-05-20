## Plan: Track C2 - AI-Brains Domain Schema & Cross-Domain Reachability

### Phase 1: AI-Brains Domain Relations
- [x] Task 1.1: Define `Turn` relation in `src/state/storage_cozo.rs` with fields: id, session_id, timestamp, project_id, summary, privacy_level.
- [x] Task 1.2: Define `Session` relation with fields: id, project_id, started_at, ended_at, turn_count, privacy_level.
- [x] Task 1.3: Define `Memory` relation with fields: id, source_turn_id, content, memory_type, privacy_level, created_at.
- [x] Task 1.4: Define `Decision` relation with fields: id, title, context_field, decision_text, consequences, source_tx_id, timestamp.
- [x] Task 1.5: Add all 4 relations to `setup_schema()` with idempotent `:create` calls (skip if already exists via `get_relations()`).
- [x] Task 1.6: Write 5 table CRUD + 1 schema existence tests — all pass.

### Phase 2: Cross-Domain Reachability Queries
- [x] Task 2.1: Define 6 query methods — `query_conversation_to_ast_via_memory`, `query_conversation_to_ast_via_decision`, `query_ast_to_conversation_via_memory`, `query_ast_to_conversation_via_session`, `query_ast_to_conversation_via_decision`, `query_ast_to_conversation_via_decision_target`.
- [x] Task 2.2: Each generates raw Datalog executed via `run_script()` — same interface AI-Brains' `CozoProxyBackend` uses.
- [x] Task 2.3: Datalog query patterns documented in `setup_schema()` comments for AI-Brains to send directly.
- [x] Task 2.4: Write 7 cross-domain tests — memory path, multi-symbol, decision path, turn/session reverse, edge-to-decision reverse, bidirectional roundtrip, negative no-match.

### Phase 3: Verification
- [x] Task 3.1: `cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo test --workspace` — 848 passed, 0 failed
- [x] Task 3.2: Existing KG queries (G1-G7) not broken by new relations.
- [x] Task 3.3: Idempotency test verifies double initialization is safe.

# Track X1 Plan: `ask` KG Fallback

## Phase 1 — Red (Failing Tests)
- [x] 1. Write integration test `tests/integration/cli_ask.rs::test_ask_uses_kg_when_semantic_empty`: init a temp repo, populate CozoDB with at least one node, leave semantic index empty, run `ask "query"`, assert response contains the "using KG" note.
- [x] 2. Write unit test for the fallback selector function: given 0 semantic results, expect KG path invoked.

## Phase 2 — Implementation
- [x] 3. In `src/commands/ask.rs`, after the semantic retrieval call, check if `results.is_empty()`. If so, and `!no_kg_fallback`, call `kg_context_search(cozo, query)`.
- [x] 4. Implement `kg_context_search(cozo: &CozoStorage, query: &str) -> Vec<String>` in `src/retrieval/kg_fallback.rs`:
  - Tokenize query into terms (split on whitespace/punctuation).
  - Run CozoDB script: `?[id, label, category] := *node{id, label, category}` and filter in Rust by any term matching `label` (case-insensitive contains).
  - Return top 20 `"[{category}] {label}"` strings.
- [x] 5. When KG fallback fires, prepend `"Note: semantic index empty — using KG text search for context\n"` to stdout before the LLM response.
- [x] 6. Add `--no-kg-fallback` flag to `AskArgs` in `src/cli.rs`.
- [x] 7. When both semantic and KG yield 0 results, inject `"No project context available."` into the system prompt.

## Phase 3 — Green + Cleanup
- [x] 8. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 9. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 10. Run `cargo fmt --all -- --check` — clean.
- [x] 11. Update `conductor/conductor.md` status to Completed.

## Plan: Track M2-2 — Retrieval, Reranking & Impact Enrichment

### Phase 1: Retrieval Query Module
- [ ] Task 1.1: Create `src/retrieval/query.rs` with `retrieve_top_k(conn: &Connection, query_vec: &[f32], entity_type: &str, model_name: &str, k: usize) -> Result<Vec<RetrievedChunk>>`.
- [ ] Task 1.2: `RetrievedChunk` struct: `entity_id: String`, `similarity: f32`, `content: String`, `heading: Option<String>`, `file_path: String`.
- [ ] Task 1.3: Load all embeddings for `entity_type` via `load_candidates()`; compute `cosine_sim` against `query_vec`; return top-k sorted descending.
- [ ] Task 1.4: For `entity_type = "doc_chunk"`, resolve `content`, `heading`, `file_path` from the `doc_chunks` table using the `entity_id`.
- [ ] Task 1.5: Write unit test: insert 5 doc_chunk embeddings with known similarity profile; `retrieve_top_k` returns correct top-3 in order.
- [ ] Task 1.6: Write unit test: `retrieve_top_k` with 0 stored embeddings returns empty vec, not error.

### Phase 2: Reranking Module
- [ ] Task 2.1: Create `src/retrieval/rerank.rs` with `rerank(base_url: &str, model: &str, query: &str, chunks: Vec<RetrievedChunk>, timeout_secs: u64) -> Result<Vec<RetrievedChunk>>`.
- [ ] Task 2.2: POST to `{base_url}/v1/rerank` with `{"model": model, "query": query, "documents": [chunk.content, ...]}`.
- [ ] Task 2.3: Parse response `results[N].relevance_score`; assign to `chunk.similarity` field (overwrite cosine score).
- [ ] Task 2.4: Sort returned chunks descending by new relevance_score.
- [ ] Task 2.5: When server is unreachable or returns error, return the original `chunks` unchanged (cosine-scored fallback).
- [ ] Task 2.6: Write unit test: mock rerank server returning reversed scores → output is reversed order.
- [ ] Task 2.7: Write unit test: rerank server unreachable → returns original chunks in original order.

### Phase 3: New ImpactPacket Fields
- [ ] Task 3.1: Add `RelevantDecision` struct to `src/impact/packet.rs`: `file_path: PathBuf`, `heading: Option<String>`, `excerpt: String` (first 200 chars), `similarity: f32`, `rerank_score: Option<f32>`.
- [ ] Task 3.2: Add `relevant_decisions: Vec<RelevantDecision>` to `ImpactPacket` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
- [ ] Task 3.3: Write unit test: `ImpactPacket` with `relevant_decisions` serializes and deserializes correctly.
- [ ] Task 3.4: Write unit test: `ImpactPacket` with empty `relevant_decisions` serializes without the field (skip_serializing_if).
- [ ] Task 3.5: Add `relevant_decisions.sort_unstable()` in `ImpactPacket::finalize()` (sort by descending similarity).
- [ ] Task 3.6: Add `relevant_decisions.clear()` in `ImpactPacket::truncate_for_context()` Phase 3, between `centrality_risks.clear()` and `logging_coverage_delta.clear()`.
- [ ] Task 3.7: Write unit test: `finalize()` sorts relevant_decisions by descending similarity.
- [ ] Task 3.8: Write unit test: `truncate_for_context()` clears relevant_decisions when budget exceeded.

### Phase 4: Impact Enrichment
- [ ] Task 4.1: In `execute_impact()` in `src/commands/impact.rs`, after all existing enrichment, compute a query vector from the current diff (changed file paths + changed symbol names joined as a single string).
- [ ] Task 4.2: Call `retrieve_top_k` for `entity_type = "doc_chunk"` to get `docs.retrieval_top_k * 3` candidates.
- [ ] Task 4.3: Call `rerank()` on the candidates; take the final top `docs.retrieval_top_k`.
- [ ] Task 4.4: Map each `RetrievedChunk` to a `RelevantDecision` (excerpt = first 200 chars of content).
- [ ] Task 4.5: Assign to `packet.relevant_decisions`.
- [ ] Task 4.6: Skip the entire enrichment step (no error) when `local_model.base_url` is empty or `doc_chunks` table has zero rows.
- [ ] Task 4.7: Write integration test: seed `doc_chunks` + `embeddings` with fixture data; run `execute_impact` on a fixture change; assert `relevant_decisions` is non-empty.
- [ ] Task 4.8: Write test: when `base_url` is empty, `execute_impact` completes without error and `relevant_decisions` is empty.

### Phase 5: Ask Context Injection
- [ ] Task 5.1: In `src/gemini/modes.rs` (or a new `src/retrieval/blend.rs`), add `format_relevant_decisions(decisions: &[RelevantDecision]) -> String` producing the fenced markdown block documented in the plan.
- [ ] Task 5.2: In `execute_ask()` in `src/commands/ask.rs`, after loading the packet, call `format_relevant_decisions` and prepend the result to the user prompt if non-empty.
- [ ] Task 5.3: Enforce budget: if adding doc context would exceed `context_window`, trim decisions from the end until it fits; log `WARN` if trimming occurs.
- [ ] Task 5.4: Write unit test: `format_relevant_decisions` with 2 decisions produces expected markdown block.
- [ ] Task 5.5: Write unit test: budget enforcement trims excess decisions rather than truncating mid-text.

### Phase 6: Final Validation
- [ ] Task 6.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 6.2: Run `cargo test --lib retrieval` and `cargo test --test doc_chunking` — all pass.
- [ ] Task 6.3: Run full `cargo test` — no regressions.
- [ ] Task 6.4: Run `changeguard index --docs` then `changeguard impact` on the changeguard repo; confirm `relevant_decisions` appears in `latest-impact.json`.

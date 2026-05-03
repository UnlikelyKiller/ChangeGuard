## Plan: Track M1-2 — Cosine Similarity, Top-K, Budget & Doctor

### Phase 1: Cosine Similarity & Top-K Retrieval
- [ ] Task 1.1: Create `src/embed/similarity.rs` with `cosine_sim(a: &[f32], b: &[f32]) -> f32`: compute dot product / (|a| * |b|); return 0.0 if either vector is zero-length.
- [ ] Task 1.2: Implement `top_k(query: &[f32], candidates: &[(String, Vec<f32>)], k: usize) -> Vec<(String, f32)>`: returns entity_ids sorted descending by cosine_sim, length = min(k, candidates.len()).
- [ ] Task 1.3: Write unit test: `cosine_sim` of identical vectors returns 1.0.
- [ ] Task 1.4: Write unit test: `cosine_sim` of orthogonal vectors returns 0.0.
- [ ] Task 1.5: Write unit test: `cosine_sim` of zero-length vector returns 0.0 (no division by zero).
- [ ] Task 1.6: Write unit test: `top_k` with 5 candidates and k=3 returns 3 results in descending score order.
- [ ] Task 1.7: Write unit test: `top_k` with k larger than candidates returns all candidates.
- [ ] Task 1.8: Implement `load_candidates(conn, entity_type: &str, model_name: &str) -> Result<Vec<(String, Vec<f32>)>>` in `src/embed/storage.rs`: loads all embeddings for a given entity_type + model_name from the DB.
- [ ] Task 1.9: Write unit test: insert 3 embeddings, call `load_candidates`, assert 3 results returned with correct entity_ids.

### Phase 2: Token Budget Enforcement
- [ ] Task 2.1: Create `src/embed/budget.rs` with `token_estimate(text: &str) -> usize` — approximate as `text.len() / 4`.
- [ ] Task 2.2: Implement `enforce_budget<'a>(parts: &[&'a str], budget: usize) -> (Vec<&'a str>, bool)` — greedy include from front until budget is reached; returns (included_parts, was_truncated).
- [ ] Task 2.3: Write unit test: `token_estimate("hello world")` returns a reasonable non-zero value.
- [ ] Task 2.4: Write unit test: `enforce_budget` with total tokens under budget includes all parts, returns `was_truncated = false`.
- [ ] Task 2.5: Write unit test: `enforce_budget` with total tokens over budget includes only fitting parts, returns `was_truncated = true`.
- [ ] Task 2.6: Write unit test: `enforce_budget` with single part exceeding budget includes that part (never truncates a single part mid-text), returns `was_truncated = false` (the part itself is returned whole).

### Phase 3: Public API in `src/embed/mod.rs`
- [ ] Task 3.1: Re-export `embed_batch` from `client`, `upsert_embedding` / `get_embedding` / `load_candidates` from `storage`, `cosine_sim` / `top_k` from `similarity`, `token_estimate` / `enforce_budget` from `budget`.
- [ ] Task 3.2: Add `embed_and_store(config: &LocalModelConfig, conn: &Connection, entity_type: &str, entity_id: &str, text: &str) -> Result<bool>` convenience function: calls `embed_batch` then `upsert_embedding`; returns `Ok(true)` if a new embedding was stored, `Ok(false)` if skipped (content hash match), `Err` on failure.
- [ ] Task 3.3: Add `embed_long_text(config: &LocalModelConfig, text: &str) -> Result<Vec<f32>>`: if text exceeds `config.dimensions * 4` tokens, split into overlapping chunks, embed each, mean-pool into a single vector.
- [ ] Task 3.4: When `config.local_model.base_url` is empty, `embed_and_store` returns `Ok(false)` immediately without making any HTTP call.
- [ ] Task 3.5: Update `embed_and_store` to use `embed_long_text` internally so callers never handle chunking.
- [ ] Task 3.6: Write unit test: `embed_and_store` with empty `base_url` returns `Ok(false)` without HTTP call.
- [ ] Task 3.7: Write unit test: `embed_and_store` with mock server stores embedding and returns `Ok(true)`.
- [ ] Task 3.8: Write unit test: `embed_and_store` called twice with same text returns `Ok(false)` on second call (no re-embed).
- [ ] Task 3.9: Write unit test: `embed_long_text` with text exceeding model limit → multiple chunks embedded, mean-pooled vector matches expected dimensions.
- [ ] Task 3.10: Write unit test: `embed_long_text` with short text → delegates to single `embed_batch` call (no chunking overhead).

### Phase 4: Doctor Extension
- [ ] Task 4.1: In `src/commands/doctor.rs` (or `src/platform/env.rs`), add a `check_local_model(config: &LocalModelConfig) -> LocalModelStatus` function that makes a test embedding call to `{base_url}/v1/embeddings`.
- [ ] Task 4.2: The test call uses a single short string ("ping") as input. If the server responds with a valid embedding, return `LocalModelStatus::Found { url, model }`. If unreachable, return `LocalModelStatus::NotReachable`.
- [ ] Task 4.3: When `base_url` is empty, return `LocalModelStatus::NotConfigured` without making any network call.
- [ ] Task 4.4: Update `print_doctor_report` in `src/output/human.rs` to display a "Local Model" row showing the status and URL.
- [ ] Task 4.5: Write unit test: `check_local_model` with empty `base_url` returns `NotConfigured`.
- [ ] Task 4.6: Write unit test: `check_local_model` with mock server returning valid JSON returns `Found`.
- [ ] Task 4.7: Write unit test: `check_local_model` with unreachable URL (connection refused) returns `NotReachable`.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features` with zero new warnings.
- [ ] Task 5.2: Run `cargo test --lib embed` — all new tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions in existing tests.
- [ ] Task 5.4: Confirm `changeguard doctor` runs without error when `local_model.base_url = ""` (no network calls made).

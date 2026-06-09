## Plan: Track M1-1 — Embedding HTTP Client & SQLite Schema

### Phase 1: Configuration Model
- [ ] Task 1.1: Add `LocalModelConfig` struct to `src/config/model.rs` with fields: `base_url`, `embedding_model`, `generation_model`, `rerank_model`, `dimensions`, `context_window`, `timeout_secs`, `prefer_local`.
- [ ] Task 1.2: Add `DocsConfig` struct with fields: `include` (Vec<String>), `chunk_tokens`, `chunk_overlap`, `retrieval_top_k`.
- [ ] Task 1.3: Add `ObservabilityConfig` struct with fields: `prometheus_url`, `service_map` (HashMap<String, String>), `log_paths`, `error_rate_threshold`, `log_lookback_secs`.
- [ ] Task 1.4: Add `ContractsConfig` struct with fields: `spec_paths` (Vec<String>).
- [ ] Task 1.5: Add `local_model`, `docs`, `observability`, `contracts` fields to root `Config` struct.
- [ ] Task 1.6: Implement `Default` for all new config structs so that an empty config reproduces existing behavior.
- [ ] Task 1.7: Write unit tests in `src/config/model.rs` verifying TOML deserialization and defaults for each new struct.
- [ ] Task 1.8: Implement `resolve_local_model_config(config: &LocalModelConfig) -> LocalModelConfig` that merges config.toml values with `std::env::var()` and `.env` file overrides (follows `read_env_key()` pattern from `src/gemini/wrapper.rs:166-189`).
- [ ] Task 1.9: Write unit test: env var `CHANGEGUARD_EMBEDDING_MODEL` overrides config.toml default.
- [ ] Task 1.10: Write unit test: `.env` file override when env var is not set.
- [ ] Task 1.11: Write unit test: config.toml explicit value takes highest priority.

### Phase 2: SQLite Migrations
- [ ] Task 2.1: Add migration (next slot after existing) in `src/state/migrations.rs` to create `embeddings` table with columns: `id`, `entity_type`, `entity_id`, `content_hash`, `model_name`, `dimensions`, `vector` (BLOB), `created_at`, and `UNIQUE(entity_type, entity_id, model_name)`.
- [ ] Task 2.2: Add index `idx_embeddings_entity ON embeddings(entity_type, entity_id)` in the same migration.
- [ ] Task 2.3: Add next migration to create `doc_chunks` table: `id`, `file_path`, `chunk_index`, `heading`, `content`, `token_count`, `UNIQUE(file_path, chunk_index)`.
- [ ] Task 2.4: Add next migration to create `api_endpoints` table: `id`, `spec_path`, `method`, `path`, `summary`, `description`, `tags`, `content_hash`, `UNIQUE(spec_path, method, path)`.
- [ ] Task 2.5: Add next migration to create `test_outcome_history` table: `id`, `diff_embedding_id` (FK to embeddings), `test_file`, `outcome` (CHECK IN 'pass','fail','skip'), `commit_hash`, `recorded_at`. Add index `idx_test_history_diff`.
- [ ] Task 2.6: Add next migration to create `observability_snapshots` table: `id`, `service_name`, `error_rate`, `latency_p99`, `recorded_at`.
- [ ] Task 2.7: Update `test_all_tables_exist` in `src/state/migrations.rs` to verify all five new tables.
- [ ] Task 2.8: Write a migration round-trip test: initialize DB, insert one row into each new table, read it back.
- [ ] Task 2.9: Add `httpmock = "0.7"` to `[dev-dependencies]` in `Cargo.toml` for mock HTTP server tests.

### Phase 3: Embedding HTTP Client
- [ ] Task 3.1: Create `src/embed/mod.rs` exporting `client`, `storage`, `similarity`, `budget` submodules.
- [ ] Task 3.2: Create `src/embed/client.rs` with function `embed_batch(base_url: &str, model: &str, texts: &[&str], timeout_secs: u64) -> Result<Vec<Vec<f32>>>`.
- [ ] Task 3.3: POST to `{base_url}/v1/embeddings` with `{"model": model, "input": texts}`, parse `data[N].embedding` as `Vec<f64>`, downcast to `f32`.
- [ ] Task 3.4: Return descriptive `Err` when server is unreachable (not panic).
- [ ] Task 3.5: Write unit test for `embed_batch` using `httpmock` to mock the `/v1/embeddings` endpoint returning a 2-element float array.
- [ ] Task 3.6: Write unit test for `embed_batch` when server returns 503 — must return `Err`, not panic.
- [ ] Task 3.7: Write unit test for `embed_batch` when server is unreachable (connection refused) — must return `Err`.

### Phase 4: Content-Addressed Embedding Storage
- [ ] Task 4.1: Create `src/embed/storage.rs` with `upsert_embedding(conn, entity_type, entity_id, text, model_name, vector: &[f32]) -> Result<()>`.
- [ ] Task 4.2: Compute `blake3(text)` as `content_hash`; skip write if stored hash matches.
- [ ] Task 4.3: On hash mismatch, replace the existing row (UPDATE or DELETE+INSERT).
- [ ] Task 4.4: Implement `get_embedding(conn, entity_type, entity_id, model_name) -> Result<Option<Vec<f32>>>`: read BLOB, deserialize as little-endian `f32` array.
- [ ] Task 4.5: Validate `dimensions` on write: if `config.local_model.dimensions * 4 != vector.len() * 4`, return `Err` with a clear message.
- [ ] Task 4.6: Write unit test: upsert embedding → get embedding → assert values match.
- [ ] Task 4.7: Write unit test: upsert same entity_id with same content_hash produces exactly one row.
- [ ] Task 4.8: Write unit test: upsert same entity_id with changed content_hash replaces the row and updates the vector.
- [ ] Task 4.9: Write unit test: dimension mismatch returns `Err`.

### Phase 5: Declare `src/embed` in `src/lib.rs`
- [ ] Task 5.1: Add `pub mod embed;` to `src/lib.rs`.
- [ ] Task 5.2: Run `cargo build` and confirm no compilation errors.
- [ ] Task 5.3: Run `cargo test --lib embed` and confirm all new tests pass.

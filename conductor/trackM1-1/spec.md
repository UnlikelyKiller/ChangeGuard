# Specification: Track M1-1 — Embedding HTTP Client & SQLite Schema

## Objective
Establish the embedding infrastructure foundation: configuration model, SQLite schema migrations, HTTP client for the local embedding model, and content-addressed vector storage. No user-visible behavior changes; this is pure infrastructure.

## Components

### 1. Configuration Model (`src/config/model.rs`)

Add four new config structs. All fields must have defaults that reproduce pre-expansion behavior when sections are absent from `.changeguard/config.toml`.

**`LocalModelConfig`**
```rust
pub struct LocalModelConfig {
    pub base_url: String,          // default: "" (disabled)
    pub embedding_model: String,   // default: "" — populated from env or .env
    pub generation_model: String,  // default: "" — populated from env or .env
    pub rerank_model: String,      // default: "" — populated from env or .env
    pub dimensions: usize,         // default: 0 — must match loaded embedding model; populated from env or .env
    pub context_window: usize,     // default: 38000
    pub timeout_secs: u64,         // default: 60
    pub prefer_local: bool,        // default: false
}
```

**Environment variable resolution (follows `gemini/wrapper.rs` pattern):**

Config values are resolved in order of priority:
1. Explicit value in `.changeguard/config.toml` (if non-empty and non-zero)
2. Environment variable (e.g. `CHANGEGUARD_EMBEDDING_MODEL`)
3. `.env` file in repo root (same `read_env_key()` pattern from `src/gemini/wrapper.rs:166-189`)
4. Code default (empty string for model names, 0 for dimensions — effectively disabled)

Env var mapping:
| Config field | Env var | `.env` key |
|---|---|---|
| `base_url` | `CHANGEGUARD_LOCAL_MODEL_URL` | `CHANGEGUARD_LOCAL_MODEL_URL` |
| `embedding_model` | `CHANGEGUARD_EMBEDDING_MODEL` | `CHANGEGUARD_EMBEDDING_MODEL` |
| `generation_model` | `CHANGEGUARD_GENERATION_MODEL` | `CHANGEGUARD_GENERATION_MODEL` |
| `rerank_model` | `CHANGEGUARD_RERANK_MODEL` | `CHANGEGUARD_RERANK_MODEL` |
| `dimensions` | `CHANGEGUARD_EMBEDDING_DIMENSIONS` | `CHANGEGUARD_EMBEDDING_DIMENSIONS` |

The binary never hardcodes specific model file names. Models are swappable by changing `.env` values — no recompile needed.

**`resolve_local_model_config()` helper:**
```rust
pub fn resolve_local_model_config(config: &LocalModelConfig) -> LocalModelConfig
```
Merges config.toml values with env/`.env` overrides following the priority chain above. Called once during config loading so all downstream code uses resolved values.

**`DocsConfig`**
```rust
pub struct DocsConfig {
    pub include: Vec<String>,       // default: []
    pub chunk_tokens: usize,        // default: 512
    pub chunk_overlap: usize,       // default: 64
    pub retrieval_top_k: usize,     // default: 5
}
```

**`ObservabilityConfig`**
```rust
pub struct ObservabilityConfig {
    pub prometheus_url: String,                    // default: ""
    pub service_map: HashMap<String, String>,      // default: {}
    pub log_paths: Vec<String>,                    // default: []
    pub error_rate_threshold: f32,                 // default: 0.05
    pub log_lookback_secs: u64,                    // default: 3600
}
```

**`ContractsConfig`**
```rust
pub struct ContractsConfig {
    pub spec_paths: Vec<String>,   // default: []
}
```

Add `local_model`, `docs`, `observability`, `contracts` fields to the root `Config` struct. All must be `#[serde(default)]`.

### 2. SQLite Migrations (`src/state/migrations.rs`)

Five new migrations, appended after the last existing migration slot (index 20 as of this writing). These will be indices 21 through 25.

**Migration 21: `embeddings`**
```sql
CREATE TABLE embeddings (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type  TEXT    NOT NULL,
    entity_id    TEXT    NOT NULL,
    content_hash TEXT    NOT NULL,
    model_name   TEXT    NOT NULL,
    dimensions   INTEGER NOT NULL,
    vector       BLOB    NOT NULL,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (entity_type, entity_id, model_name)
);
CREATE INDEX idx_embeddings_entity ON embeddings (entity_type, entity_id);
```

**Migration 22: `doc_chunks`**
```sql
CREATE TABLE doc_chunks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path   TEXT    NOT NULL,
    chunk_index INTEGER NOT NULL,
    heading     TEXT,
    content     TEXT    NOT NULL,
    token_count INTEGER NOT NULL,
    UNIQUE (file_path, chunk_index)
);
```

**Migration 23: `api_endpoints`**
```sql
CREATE TABLE api_endpoints (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    spec_path    TEXT NOT NULL,
    method       TEXT NOT NULL,
    path         TEXT NOT NULL,
    summary      TEXT,
    description  TEXT,
    tags         TEXT,
    content_hash TEXT NOT NULL,
    UNIQUE (spec_path, method, path)
);
```

**Migration 24: `test_outcome_history`**
```sql
CREATE TABLE test_outcome_history (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    diff_embedding_id INTEGER NOT NULL REFERENCES embeddings(id),
    test_file         TEXT    NOT NULL,
    outcome           TEXT    NOT NULL CHECK (outcome IN ('pass', 'fail', 'skip')),
    commit_hash       TEXT,
    recorded_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_test_history_diff ON test_outcome_history (diff_embedding_id);
```

**Migration 25: `observability_snapshots`**
```sql
CREATE TABLE observability_snapshots (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    service_name TEXT NOT NULL,
    error_rate   REAL,
    latency_p99  REAL,
    recorded_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 3. Embedding HTTP Client (`src/embed/client.rs`)

POST to `{base_url}/v1/embeddings` with body:
```json
{"model": "<embedding_model>", "input": ["text1", "text2", ...]}
```
Set `Content-Type: application/json`. Parse `data[N].embedding` (array of f64) and downcast to `f32`. Use `ureq` AgentBuilder pattern matching `src/gemini/wrapper.rs` (configurable timeouts via `timeout_read`/`timeout_write`).

Batch size: up to 32 texts per request. Callers providing more must split batches themselves.

### 4. Content-Addressed Storage (`src/embed/storage.rs`)

Vectors are stored as raw little-endian `f32` bytes in the `vector` BLOB column. The `content_hash` is `blake3::hash(text.as_bytes()).to_hex().to_string()`.

`upsert_embedding` logic:
1. Compute `content_hash`.
2. Query for existing row with `(entity_type, entity_id, model_name)`.
3. If row exists and `content_hash` matches: no-op, return `Ok(())`.
4. If row exists and hash differs: UPDATE `content_hash`, `vector`, `created_at`.
5. If no row: INSERT.

## Constraints & Guidelines

- **TDD**: Write failing tests before implementing each function.
- **Graceful degradation**: When `base_url` is empty, `embed_batch` returns an empty `Vec` (or `Ok(None)`) without making any network call.
- **No production unwraps**: All `Result` propagation must use `?` or explicit `match`.
- **Windows paths**: `entity_id` for file-based entities uses forward slashes. Normalize on input.
- **No new native deps**: Use `ureq` (already present) and `blake3` (already present).
- **Test isolation**: All tests that touch SQLite use `tempfile::tempdir()` for DB path.

## Deviations from `docs/observability-plan.md`

| Deviation | Reason | Plan reference |
|---|---|---|
| Model names/dimensions are empty defaults resolved from env/`.env` | Models are swappable without recompiling. Follows the existing `GEMINI_API_KEY` `.env` pattern in `src/gemini/wrapper.rs:166-189`. | Plan §6: hardcoded defaults `text-embedding-nomic-embed-text`, `qwen3-9b-6q`, `bge-reranker-v2-m3`, `dimensions = 768` |
| Migration numbers are "next available slots" not M-10..M-14 | Existing codebase already has 21 migrations (indices 0-20). New tables go at indices 21-25. | Plan §5.1: used M-10..M-14 which collide with Ledger M11-M14 |

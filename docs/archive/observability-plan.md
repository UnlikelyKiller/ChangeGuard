# ChangeGuard Observability & Intelligence Expansion Plan

## Overview

This document is the implementation roadmap for extending ChangeGuard's coverage across four currently under-served engineering onboarding dimensions:

1. **Observability** â€” live system signals (metrics, logs, traces) correlated to code changes
2. **API & Integration** â€” contract awareness, external dependency surface, secret dependency mapping
3. **Architecture & Design** â€” ADR retrieval, design doc awareness, semantic architectural context
4. **Process & Workflow** â€” semantically-driven test prediction, PR generation, flaky test tracking

The expansion is structured around a central new capability: a **local embedding and retrieval pipeline** backed by the existing SQLite ledger and the user's local inference stack (llama-server + Qwen 3.5 9B 6Q, embedding model, reranking model).

This plan is written for implementation by AI coding agents and humans working together. Each phase has stable boundaries, explicit edge cases, and verification gates. Phases are ordered so the embedding infrastructure (Phase M1) must be complete before any dependent phase begins.

---

## 0. Executive Summary

The six phases in this plan extend ChangeGuard from a **change-aware risk analyzer** to a **context-aware engineering intelligence layer** that understands the live system, the architectural record, and the semantic relationships between code, tests, API contracts, and observed behavior.

Key additions compared to the current implementation:

1. **Local vector store** in the existing SQLite ledger â€” no external services required.
2. **Document intelligence** â€” ADRs, design docs, and README sections are indexed and retrieved by semantic similarity to the current change.
3. **Local model backend** â€” Qwen 3.5 9B served by llama-server as a zero-quota, zero-latency alternative to Gemini for `ask` and impact enrichment.
4. **Semantic test prediction** â€” past diffs and their test outcomes are embedded to predict which tests are most likely to fail on the current change.
5. **Observability integration** â€” Prometheus metrics and local log files are queried and embedded to surface live system signals relevant to the current change.
6. **OpenAPI contract indexing** â€” API specs are parsed, embedded, and matched to changed files to flag public contract risk.

All six phases are additive. Existing commands are not broken. Each phase degrades gracefully when its infrastructure is absent.

---

## 1. Product Intent

This expansion does not change ChangeGuard's core identity.

ChangeGuard remains a **local-first change intelligence and verification orchestration CLI**. It does not become an autonomous agent, a monitoring platform, or a documentation generator.

The new phases extend its input surface. Instead of reasoning only about what changed in the source tree, it can now reason about:

* what the live system was doing before the change
* what the architectural record says about the changed area
* what past changes that look like this one caused to break
* what external contracts the changed code is party to

These signals flow into the existing `ImpactPacket`, risk scorer, and `ask` context. No new output formats are required. The packet gets richer; the commands that consume it work the same.

---

## 2. Core Implementation Principles

### 2.1 Non-Negotiable Principles

All principles from `Plan.md Â§2.1` carry forward. Additional principles for this expansion:

1. **Embedding infrastructure is opt-in.** If no local model is configured, all embedding-dependent features silently degrade to their current behavior. The binary never fails to start because a model is absent.
2. **Local inference first for new AI features.** New AI-assisted features default to the local model stack, not Gemini, to avoid quota dependency for high-frequency operations.
3. **Vector storage in SQLite.** No new storage backend. Embeddings are stored as `BLOB` columns in `ledger.db` alongside existing state. This preserves the single-store model and `changeguard reset` rebuild semantics.
4. **Cosine similarity in Rust by default.** `sqlite-vec` is an optional acceleration path, not a requirement. The default implementation computes dot products in Rust over `BLOB`-stored `f32` arrays. This avoids native extension complexity on Windows.
5. **Chunking is deterministic.** Document chunking (for ADRs, specs, log lines) uses heading-boundary splitting with a 512-token cap and a 64-token overlap. Same input always produces the same chunks.
6. **No embedding on hot path.** Embedding generation happens in `index` and dedicated pre-compute steps, never during `scan` or real-time `verify` execution.
7. **38k context window is a hard budget.** All context assembly for local model queries must fit within 38,000 tokens. Context builders must truncate and log a warning rather than silently exceed the budget.

### 2.2 Explicit Anti-Goals for This Expansion

* No background daemon or always-on embedding process.
* No embedding of production secrets, credentials, or `.env` file contents.
* No automatic telemetry upload to any external system.
* No retraining or fine-tuning of models.
* No vector similarity search as a replacement for deterministic analysis â€” it is supplemental only.
* No mandatory Prometheus or logging platform â€” observability integration is always optional.

---

## 3. Architecture Boundaries

The existing boundaries from `Plan.md Â§3` are preserved. This expansion adds:

11. **Embedding generation and storage** (`src/embed/`)
12. **Document chunking and doc index** (`src/docs/`)
13. **Local model client** (`src/local_model/`)
14. **Observability signal fetching** (`src/observability/`)
15. **API contract parsing and indexing** (`src/contracts/`)
16. **Semantic retrieval and reranking** (`src/retrieval/`)

These subsystems are **not collapsed into existing modules**. They produce enrichment data that flows into the impact packet and `ask` context via clearly named fields.

---

## 4. New Repository Layout

New files and directories added by this plan. Existing layout from `Plan.md Â§4` is unchanged.

```text
src/
â”œâ”€â”€ embed/
â”‚   â”œâ”€â”€ mod.rs              # public API: embed_text(), embed_batch(), cosine_sim()
â”‚   â”œâ”€â”€ client.rs           # HTTP client for local model /v1/embeddings endpoint
â”‚   â”œâ”€â”€ storage.rs          # read/write embeddings to SQLite BLOB columns
â”‚   â”œâ”€â”€ similarity.rs       # cosine similarity, top-k retrieval over stored vectors
â”‚   â””â”€â”€ budget.rs           # token counting, context window enforcement
â”œâ”€â”€ docs/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ crawler.rs          # walks docs/, adr/, README, CHANGEGUARD.md
â”‚   â”œâ”€â”€ chunker.rs          # heading-boundary + token-cap splitting
â”‚   â””â”€â”€ index.rs            # orchestrates crawl â†’ chunk â†’ embed â†’ store
â”œâ”€â”€ local_model/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ client.rs           # OpenAI-compatible HTTP client (completions + embeddings)
â”‚   â”œâ”€â”€ context.rs          # context assembly: diff + retrieved chunks + packet
â”‚   â””â”€â”€ rerank.rs           # reranking model client
â”œâ”€â”€ observability/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ prometheus.rs       # PromQL query client
â”‚   â”œâ”€â”€ log_scanner.rs      # read local log files, chunk lines
â”‚   â””â”€â”€ signal.rs           # ObservabilitySignal type, risk elevation logic
â”œâ”€â”€ contracts/
â”‚   â”œâ”€â”€ mod.rs
â”‚   â”œâ”€â”€ parser.rs           # OpenAPI 3.x / Swagger 2.x YAML/JSON parser
â”‚   â”œâ”€â”€ index.rs            # endpoint embedding and storage
â”‚   â””â”€â”€ matcher.rs          # match changed files to affected endpoints
â””â”€â”€ retrieval/
    â”œâ”€â”€ mod.rs
    â”œâ”€â”€ query.rs            # retrieve top-k by embedding similarity
    â”œâ”€â”€ rerank.rs           # rerank candidates using reranking model
    â””â”€â”€ blend.rs            # blend semantic score with rule-based score

tests/
â”œâ”€â”€ embed_storage.rs
â”œâ”€â”€ doc_chunking.rs
â”œâ”€â”€ local_model_context.rs
â”œâ”€â”€ semantic_test_prediction.rs
â”œâ”€â”€ observability_signal.rs
â””â”€â”€ contract_matching.rs
```

---

## 5. New State Model

### 5.1 New SQLite Tables

These tables are added via numbered migrations in `src/state/migrations.rs`.

```sql
-- Migration M-10: Embedding storage
CREATE TABLE embeddings (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type   TEXT    NOT NULL,
    entity_id     TEXT    NOT NULL,
    content_hash  TEXT    NOT NULL,
    model_name    TEXT    NOT NULL,
    dimensions    INTEGER NOT NULL,
    vector        BLOB    NOT NULL,
    created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (entity_type, entity_id, model_name)
);
CREATE INDEX idx_embeddings_entity ON embeddings (entity_type, entity_id);

-- Migration M-11: Document chunks
CREATE TABLE doc_chunks (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path    TEXT    NOT NULL,
    chunk_index  INTEGER NOT NULL,
    heading      TEXT,
    content      TEXT    NOT NULL,
    token_count  INTEGER NOT NULL,
    UNIQUE (file_path, chunk_index)
);

-- Migration M-12: API endpoints
CREATE TABLE api_endpoints (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    spec_path    TEXT    NOT NULL,
    method       TEXT    NOT NULL,
    path         TEXT    NOT NULL,
    summary      TEXT,
    description  TEXT,
    tags         TEXT,
    content_hash TEXT    NOT NULL,
    UNIQUE (spec_path, method, path)
);

-- Migration M-13: Test outcome history for semantic prediction
CREATE TABLE test_outcome_history (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    diff_embedding_id   INTEGER NOT NULL REFERENCES embeddings(id),
    test_file           TEXT    NOT NULL,
    outcome             TEXT    NOT NULL CHECK (outcome IN ('pass', 'fail', 'skip')),
    commit_hash         TEXT,
    recorded_at         TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_test_history_diff ON test_outcome_history (diff_embedding_id);

-- Migration M-14: Observability snapshots
CREATE TABLE observability_snapshots (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    service_name  TEXT    NOT NULL,
    error_rate    REAL,
    latency_p99   REAL,
    recorded_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
```

### 5.2 Rebuild Semantics

All embedding tables are derived state. `changeguard reset --embeddings` drops and rebuilds them. `changeguard reset` (full) drops all new tables. Embeddings that cannot be rebuilt (e.g. old diff embeddings with no source commit to re-derive from) are lost on reset â€” this is acceptable because they are historical optimization data, not authoritative state.

---

## 6. New Configuration Sections

Added to `.changeguard/config.toml`:

```toml
[local_model]
# Base URL for the llama-server OpenAI-compatible API.
# Set to empty string to disable all local model features.
base_url         = "http://localhost:8080"
embedding_model  = "text-embedding-nomic-embed-text"
generation_model = "qwen3-9b-6q"
rerank_model     = "bge-reranker-v2-m3"
dimensions       = 768         # must match the embedding model's output
context_window   = 38000       # tokens; hard budget for context assembly
timeout_secs     = 60

[docs]
# Directories and files to include in the document index.
# Relative to repo root. Glob patterns supported.
include = ["docs/**/*.md", "adr/**/*.md", "README.md", "CHANGEGUARD.md"]
# Maximum chunk size in tokens before splitting.
chunk_tokens     = 512
chunk_overlap    = 64
# How many retrieved doc chunks to attach to impact packet.
retrieval_top_k  = 5

[observability]
# Prometheus base URL. Leave empty to disable metrics integration.
prometheus_url   = ""
# Map from source path prefix to service name in Prometheus.
# service_map = { "src/payments/" = "payments-svc", "src/auth/" = "auth-svc" }
service_map      = {}
# Local log file globs to scan for anomalies.
log_paths        = []
# Error rate above this threshold elevates risk by one tier.
error_rate_threshold = 0.05
# How far back to look for log lines (seconds).
log_lookback_secs    = 3600

[contracts]
# Paths to OpenAPI 3.x or Swagger 2.x spec files.
# Relative to repo root. Glob patterns supported.
spec_paths       = []
```

---

## 7. New Impact Packet Fields

The following fields are added to `ImpactPacket` (in `src/impact/packet.rs`). All are `Option<T>` or empty `Vec` by default so existing callers are unaffected.

```rust
/// Semantically relevant architecture documents retrieved for this change.
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub relevant_decisions: Vec<RelevantDecision>,

/// API endpoints potentially affected by this change.
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub affected_contracts: Vec<AffectedContract>,

/// Live system signals at the time of impact analysis.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub observability: Option<ObservabilitySignal>,
```

Supporting types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelevantDecision {
    pub file_path: PathBuf,
    pub heading: Option<String>,
    pub excerpt: String,       // first 200 chars of chunk
    pub similarity: f32,
    pub rerank_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AffectedContract {
    pub spec_path: PathBuf,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservabilitySignal {
    pub services: Vec<ServiceSignal>,
    pub log_anomaly_count: usize,
    pub risk_elevation: Option<String>,  // reason string if signal elevated risk
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceSignal {
    pub service_name: String,
    pub error_rate: Option<f32>,
    pub latency_p99_ms: Option<f32>,
    pub above_threshold: bool,
}
```

---

## 8. Threat Model Extensions

### 8.1 Embedding Safety

* Embeddings are computed over source code and documentation text only. They are never computed over `.env` files, secrets files, or any file matching the existing protected-path list.
* The embedding HTTP request goes to `localhost` only. The config validator rejects non-localhost URLs with a warning (the user must explicitly override this to point at a remote model server).
* Embedding content is not included in reports exported to Gemini or any remote service.

### 8.2 Observability Safety

* Prometheus queries use read-only PromQL. No write endpoints are called.
* Log file reading is bounded by `log_lookback_secs` and a 10 MB per-file read cap.
* Log content is not forwarded to Gemini. It is embedded locally and the log anomaly count + similarity score is included in the packet; raw log content is not.

### 8.3 Contract Safety

* OpenAPI spec parsing is sandboxed â€” a malformed spec causes a warning and skips that file; it does not abort the impact run.
* Spec files are treated as read-only inputs. ChangeGuard never writes to or modifies spec files.

### 8.4 Local Model Safety

* All local model requests go to the configured `base_url`. If the server is unreachable, the feature degrades gracefully with a `WARN` log; it does not fail the command.
* Request bodies are subject to the same sanitizer (`src/gemini/sanitize.rs`) as Gemini prompts.
* The 38k context window cap is enforced before sending. Exceeding the cap truncates with a warning.

---

## 9. High-Level Delivery Sequence

Phases must be completed in this order due to infrastructure dependencies:

1. **M1** â€” Local embedding infrastructure (all other phases depend on this)
2. **M2** â€” Document intelligence (depends on M1 embedding storage and retrieval)
3. **M3** â€” Local model `ask` backend (depends on M1 client and embedding for context assembly)
4. **M4** â€” Semantic test prediction (depends on M1 storage and M3 client)
5. **M5** â€” Metrics and log observability (depends on M1 for log embedding)
6. **M6** â€” OpenAPI contract indexing (depends on M1 for endpoint embedding)

M5 and M6 are independent of each other after M1 is complete.

---

## Phase M1: Local Embedding Infrastructure

### Objective

Establish the embedding generation, storage, and retrieval foundation used by all subsequent phases. No user-facing behavior changes in M1 â€” this is infrastructure only.

### Deliverables

* `src/embed/client.rs` â€” HTTP client for the llama-server `/v1/embeddings` endpoint
* `src/embed/storage.rs` â€” read/write `f32` vectors as SQLite `BLOB`s
* `src/embed/similarity.rs` â€” cosine similarity, top-k over stored vectors
* `src/embed/budget.rs` â€” token budget enforcement for context assembly
* `src/embed/mod.rs` â€” public API
* SQLite migrations M-10 through M-14 (schema only; tables are empty until later phases fill them)
* `[local_model]` config section parsing and validation
* `changeguard doctor` extended: checks `local_model.base_url` reachability and reports model availability

### Functional Requirements

**Embedding client:**
* POST to `{base_url}/v1/embeddings` with `{"model": "{embedding_model}", "input": ["..."]}`
* Accept `OpenAI`-compatible response: `data[0].embedding` as `Vec<f64>`, downcast to `f32`
* Support batch requests: send up to 32 texts per request, respecting server capability
* Configurable timeout via `local_model.timeout_secs`
* Return `Err` with actionable message when server is unreachable, not panic

**Storage:**
* Store vectors as raw little-endian `f32` bytes in a `BLOB` column
* Content-addressed by `blake3(text)` â€” re-embedding the same text with the same model is a no-op
* Invalidation: if `content_hash` differs from stored hash, replace the vector
* Dimensions sanity check on write: if `dimensions` field does not match `vector.len() / 4`, return `Err`

**Similarity:**
* `cosine_sim(a: &[f32], b: &[f32]) -> f32` â€” normalized dot product
* `top_k(query: &[f32], candidates: &[(entity_id, vector)], k: usize) -> Vec<(entity_id, f32)>` â€” returns sorted descending by similarity
* Candidate loading is done in Rust from SQLite rows, not in SQL, because SQLite has no vector operator by default

**Budget enforcement:**
* `token_estimate(text: &str) -> usize` â€” approximate by `text.len() / 4` (fast heuristic, no tokenizer dependency)
* `enforce_budget(parts: &[&str], budget: usize) -> Vec<&str>` â€” greedily includes parts until budget is reached, logs a warning if truncation occurs

**Doctor extension:**
* Try a test embedding call to `base_url` during `changeguard doctor`
* Report: `Local Model: Found ({base_url}, {embedding_model})` or `Local Model: Not reachable`
* Failure here is a warning, not an error

### Edge Cases

* llama-server not running: all embed calls return `Ok(None)` with a `WARN` log; no crash
* Model name mismatch (server has different model loaded): server returns 404 or error body; surface as `WARN` with the error message from the server
* Dimension mismatch between config and returned vector: hard `Err` with clear message instructing the user to update `local_model.dimensions`
* Very large text (>8192 tokens): truncate with `budget.rs` before sending; log the truncation
* Concurrent `index` runs: use the same SQLite write lock already used by other index operations; no new locking needed
* Embedding for a file that is later deleted: stale embedding rows are acceptable; they are ignored if their `entity_id` no longer appears in `frequency_map`
* Windows path normalization: all `entity_id` values for files must use forward slashes to match the rest of the codebase

### Acceptance Criteria

* `cargo test --test embed_storage` passes with a mock HTTP server
* `changeguard doctor` reports local model status correctly when server is up and when server is down
* Embedding a file twice with the same content does not produce two rows
* Changing a file's content and re-indexing replaces the vector (content hash changes)
* Binary builds and passes CI on both Windows and Ubuntu with `local_model.base_url = ""`

### Verification Gate

* Unit tests: `cosine_sim`, `top_k`, `token_estimate`, `enforce_budget`
* Integration tests: `embed_storage.rs` with a mock HTTP server (use `mockito` or `httpmock`)
* CI: build must pass with no active llama-server (`base_url = ""` in test config)

---

## Phase M2: Document Intelligence

### Objective

Index architecture documents (ADRs, design docs, README sections) into the embedding store and surface the most relevant documents for any given code change in the impact packet.

### Deliverables

* `src/docs/crawler.rs` â€” walks configured include paths, respects `.gitignore`
* `src/docs/chunker.rs` â€” heading-boundary split with token cap
* `src/docs/index.rs` â€” orchestrates crawl â†’ chunk â†’ embed â†’ store
* `changeguard index --docs` flag (or standalone `changeguard docs index` subcommand)
* `src/retrieval/query.rs` â€” retrieve top-k doc chunks by similarity to a query vector
* `src/retrieval/rerank.rs` â€” rerank candidates via reranking model
* Impact enrichment: `relevant_decisions` field populated in `ImpactPacket`
* `changeguard ask` context extended: retrieved doc chunks prepended to user prompt

### Functional Requirements

**Crawler:**
* Walk paths matching `docs.include` globs relative to repo root
* Skip files matching `.gitignore` rules (reuse existing gitignore logic)
* Read only `.md`, `.txt`, `.rst`, `.adoc` files; skip binaries silently
* Record `file_path` and `last_modified` for incremental re-indexing

**Chunker:**
* Split Markdown on `##` and `###` headings; preserve the heading text as `heading` field
* If a section exceeds `chunk_tokens`, split on paragraph boundaries (double newline)
* Minimum chunk size: 50 tokens; discard smaller chunks
* Each chunk stores: `file_path`, `chunk_index`, `heading`, `content`, `token_count`
* Chunking is deterministic: same file always produces the same chunks

**Incremental indexing:**
* Compare `blake3(chunk_content)` against stored `content_hash`
* Only re-embed chunks whose content has changed
* Orphaned chunk rows (file deleted or section removed) are deleted

**Retrieval:**
* On `execute_impact()`, embed the current diff summary (changed file paths + changed symbol names, joined as text)
* Query `embeddings` for `entity_type = 'doc_chunk'`, compute cosine similarity, take top `docs.retrieval_top_k * 3` candidates
* Send candidates to reranking model: POST `{base_url}/v1/rerank` with query = diff summary text, documents = chunk contents
* If reranking model is unavailable, use cosine similarity score directly
* Final top-k by rerank score (or cosine score) is attached to packet as `relevant_decisions`

**Reranking model API:**
* Expected endpoint: `POST /v1/rerank`
* Request: `{"model": "{rerank_model}", "query": "...", "documents": ["...", ...]}`
* Response: `{"results": [{"index": 0, "relevance_score": 0.92}, ...]}`
* This matches the Cohere-compatible reranking API that llama.cpp's reranking serves

**Context injection in `ask`:**
* Retrieved doc chunks are prepended to the user prompt in a fenced block:
  ```
  ## Relevant Architecture Documents
  ### {heading} ({file_path})
  {excerpt}
  ---
  ```
* Token budget is enforced: doc context + diff + packet summary must fit in `context_window`
* If budget is exceeded, doc context is trimmed first (it is supplemental)

### Edge Cases

* No docs directory exists: `changeguard index --docs` completes silently with a note
* ADR files with no headings: treat entire file as one chunk
* Very large single-section ADR (>2000 tokens): split on paragraph boundary; log if any paragraph itself exceeds `chunk_tokens`
* Reranking model not loaded: fall back to cosine similarity; log `WARN` once per session
* Diff query is empty (clean working tree): use recent commit message as query fallback
* Retrieved doc chunks are from an outdated index (file changed since last index run): stale excerpts are acceptable; they are supplemental context, not authoritative
* Circular doc references (one ADR links to another): crawler does not follow links; each file is indexed independently

### Acceptance Criteria

* `changeguard index --docs` completes without error on a repo with `docs/` and `adr/` directories
* `changeguard impact` on a changed file that matches an ADR topic includes at least one `relevant_decisions` entry
* Re-running `changeguard index --docs` without changing any doc files produces zero new embedding writes (verified by counting rows before/after)
* With reranking model unavailable, `impact` still completes and populates `relevant_decisions` using cosine similarity
* `changeguard ask` output includes retrieved doc context before the Gemini/local model response

### Verification Gate

* Unit tests: `chunker.rs` â€” heading split, paragraph split, minimum chunk enforcement
* Unit tests: `doc_crawler.rs` â€” gitignore skip, file type filter, incremental hash check
* Integration test: `doc_chunking.rs` â€” index a fixture docs directory, assert chunk count and content
* Integration test: embed + retrieve round-trip with mock HTTP server

---

## Phase M3: Local Model `ask` Backend

### Objective

Route `changeguard ask` to the local Qwen 3.5 9B model when the local backend is configured, eliminating Gemini API dependency for routine queries. The local model backend is available for all four `ask` modes.

### Deliverables

* `src/local_model/client.rs` â€” OpenAI-compatible completions client for llama-server
* `src/local_model/context.rs` â€” context assembly: diff + retrieved doc chunks + impact packet summary
* `src/local_model/rerank.rs` â€” shared reranking client (also used by Phase M2)
* `changeguard ask --backend local` flag
* Auto-selection: if `local_model.base_url` is set and Gemini key is absent, default to local
* `changeguard config verify` subcommand: extended to report which backend will be used for `ask`

### Functional Requirements

**Client:**
* POST to `{base_url}/v1/chat/completions`
* Request: `{"model": "{generation_model}", "messages": [{"role": "system", "content": "..."}, {"role": "user", "content": "..."}], "stream": false}`
* Response: parse `choices[0].message.content`
* Timeout: `local_model.timeout_secs`
* No retry logic by default (local server is expected stable); single retry on `503`

**Context assembly:**
* System prompt: same prompts as the Gemini modes (`src/gemini/modes.rs`) â€” modes are reusable
* User prompt composition order (descending priority in budget):
  1. User's query text (never truncated)
  2. Impact packet summary (risk level, risk reasons, changed files â€” compact form, ~500 tokens)
  3. Retrieved doc chunks from Phase M2 (up to `docs.retrieval_top_k` chunks)
  4. Temporal coupling summary (top 5 couplings)
  5. Hotspot summary (top 5 hotspots)
  6. Full diff (if `review-patch` mode and diff is available)
* Budget enforcement via `src/embed/budget.rs` â€” hard cap at `context_window - 500` tokens (reserve 500 for generation headroom)

**Backend routing:**
* `--backend local` flag forces local model
* `--backend gemini` flag forces Gemini (existing behavior)
* Default (no flag):
  1. If `local_model.base_url` is non-empty AND Gemini key absent â†’ local
  2. If `local_model.base_url` is non-empty AND `local_model.prefer_local = true` â†’ local
  3. Otherwise â†’ Gemini (existing `wrapper.rs` path)
* `prefer_local` is a new boolean field in `[local_model]` config (default: `false`)

**Output:**
* Same output format as Gemini: `Gemini Response:` header is replaced with `Local Model Response:` when using local backend
* All existing display functions in `src/output/human.rs` are unaffected

### Edge Cases

* llama-server not running when `--backend local` is forced: return `Err` with message "Local model server is not reachable at {base_url}. Start llama-server or use `--backend gemini`."
* Context assembly exceeds budget: truncate lowest-priority components first (hotspot/coupling summary, then doc chunks); never truncate the user query or the impact packet risk reasons
* `review-patch` mode with a very large diff: diff is the last component added; it is the first truncated
* Generation model not loaded in llama-server (different model is active): server returns 404 on the model name; surface as `Err` with the server's error message and advice to check the active model
* Windows path in impact packet (backslashes): paths are normalized to forward slash before inclusion in prompts to avoid confusion in model output

### Acceptance Criteria

* `changeguard ask --backend local "what is the risk?"` succeeds when llama-server is running with Qwen 3.5 loaded
* `changeguard ask --backend local` fails with a clear message when llama-server is not running
* Context assembly for a typical impact packet (10 changed files, 5 hotspots, 3 doc chunks) fits within 38k tokens
* Auto-selection correctly prefers local model when `prefer_local = true` and falls back to Gemini when local is unreachable
* All four modes (`analyze`, `suggest`, `review-patch`, `narrative`) work with `--backend local`

### Verification Gate

* Unit tests: `local_model_context.rs` â€” context assembly order, budget truncation behavior
* Unit tests: backend routing logic (local/gemini selection given various config states)
* Integration test: mock llama-server response, verify prompt is correctly assembled and response is printed
* CI: passes with `base_url = ""` (local model disabled, falls through to Gemini path)

---

## Phase M4: Semantic Test Prediction

### Objective

Supplement the existing rule-based test predictor with a semantic layer: embed past diffs and their test outcomes, then rank test files by how similar the current change is to past changes where those tests failed.

### Deliverables

* `src/verify/semantic_predictor.rs` â€” semantic prediction layer
* `src/verify/predict.rs` extended â€” blends semantic score with existing rule-based score
* `execute_verify()` extended â€” records test outcomes into `test_outcome_history` after each run
* New weight in `[verify]` config: `semantic_weight` (default: `0.3`, range 0.0â€“1.0)
* `changeguard verify --explain` flag â€” prints prediction rationale per test file

### Functional Requirements

**Outcome recording:**
* After `execute_verify()` completes, for each test file that ran:
  * Compute embedding of the current diff (file paths + changed symbols as text, same as Phase M2 query)
  * Store `(diff_embedding_id, test_file, outcome, commit_hash)` in `test_outcome_history`
* If embedding server is unavailable, skip recording without error

**Semantic prediction:**
* On `execute_verify()` start, before running tests:
  * Embed the current diff
  * Retrieve top-30 most similar past diff embeddings from `test_outcome_history`
  * For each test file appearing in retrieved history, compute:
    `semantic_fail_rate = count(outcome='fail') / count(*)`
  * Weight by similarity score: `weighted_score = similarity * semantic_fail_rate`
  * Normalize to [0.0, 1.0]

**Score blending in predictor:**
* Existing rule-based score: `rule_score`
* Semantic score: `semantic_score` (0.0 if history is empty or embedding unavailable)
* Blended: `final_score = (1.0 - semantic_weight) * rule_score + semantic_weight * semantic_score`
* Tests with no rule-based coverage but high semantic score are still surfaced

**`--explain` output:**
```
Test priority rationale:
  tests/auth.rs       rule: 0.80  semantic: 0.72  final: 0.78
    Semantic basis: 3 of 4 similar past changes caused failures in this test
  tests/ledger.rs     rule: 0.60  semantic: 0.00  final: 0.42
    Semantic basis: insufficient history (< 5 samples)
```

**Cold start behavior:**
* With fewer than 5 history records for a diff embedding neighborhood, `semantic_score = 0.0` and the rule-based score is used exclusively
* A note is printed: `Semantic prediction: warming up (N/50 history records)`

### Edge Cases

* Embedding unavailable: `semantic_score = 0.0` for all tests, rule-based score used unchanged
* Test file renamed: old history entries are not updated (they retain the old path); they naturally decay in influence as newer entries accumulate
* Test suite produces no output files (no structured result format): outcome is inferred from exit code â€” exit 0 is `pass`, non-zero is `fail`
* Same diff content committed multiple times (e.g. reverts): multiple history entries with the same diff hash are allowed; they contribute proportionally to the weighted score
* `semantic_weight = 0.0`: semantic predictor is fully disabled; behavior is identical to current
* `semantic_weight = 1.0`: rule-based predictor is fully disabled; not recommended; allowed with a warning

### Acceptance Criteria

* After 10 runs of `changeguard verify` on a repo with a consistent failing test, that test appears at rank 1 in predictions for similar changes
* `changeguard verify --explain` prints rationale for every predicted test file
* `semantic_weight = 0.0` produces identical test ordering to the current predictor
* Outcome recording does not extend verify runtime by more than 500ms
* History rows accumulate correctly in `test_outcome_history` across multiple verify runs

### Verification Gate

* Unit tests: `semantic_test_prediction.rs` â€” weighted score computation, cold start behavior, blending
* Unit tests: outcome recording writes correct rows
* Integration test: seed `test_outcome_history` with fixture data, run prediction, assert test order matches expectation
* CI: passes with embedding server unavailable

---

## Phase M5: Metrics and Log Observability

### Objective

Pull live system signals from Prometheus and local log files at impact analysis time. Signals that exceed configured thresholds elevate the risk tier and appear in the impact packet as `observability`.

### Deliverables

* `src/observability/prometheus.rs` â€” PromQL instant query client
* `src/observability/log_scanner.rs` â€” read local log files, emit chunks for embedding
* `src/observability/signal.rs` â€” `ObservabilitySignal` computation and risk elevation logic
* Impact enrichment: `observability` field populated in `ImpactPacket`
* `changeguard scan --impact` extended: runs observability fetch in parallel with other enrichment
* `changeguard ask` context extended: `ObservabilitySignal` summary included in user prompt

### Functional Requirements

**Prometheus integration:**
* For each changed file, look up matching service name(s) from `observability.service_map`
* For each matched service, execute two PromQL instant queries:
  * Error rate: `rate(http_requests_total{job="{service}", status=~"5.."}[5m]) / rate(http_requests_total{job="{service}"}[5m])`
  * Latency: `histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{job="{service}"}[5m]))`
* Queries use the Prometheus HTTP API: `GET {prometheus_url}/api/v1/query?query={encoded_promql}`
* Timeout: 5 seconds; failure returns `None` per service, not an error
* Results stored in `observability_snapshots` for history; used to detect trend (current vs. 1h-ago snapshot)

**Risk elevation:**
* If any service's `error_rate > observability.error_rate_threshold`:
  * `ObservabilitySignal.risk_elevation = Some("Service {name} error rate {rate:.1%} exceeds threshold")`
  * Risk tier is elevated by one level (Low â†’ Medium, Medium â†’ High) in the final packet risk score
  * The elevation reason is added to `packet.risk_reasons`
* Latency is informational only; it does not affect the risk tier in this phase

**Log scanning:**
* Read files matching `observability.log_paths` globs, newest bytes first, up to `log_lookback_secs` seconds back
* Hard cap: 10 MB total across all log files per run
* Chunk log lines into groups of 20 lines (preserving temporal proximity)
* Embed each chunk using the local embedding model
* Compute cosine similarity between each log chunk and the current diff embedding
* Log chunks with similarity > 0.6 are flagged as anomalies; count stored in `ObservabilitySignal.log_anomaly_count`
* Raw log content is never included in the packet or sent to any remote API

**Observability in `ask` context:**
* If `observability` is non-null in the packet, include a compact summary in the user prompt:
  ```
  ## Live System Signals
  Service payments-svc: error_rate=3.2%, latency_p99=450ms (above threshold)
  Log anomalies: 4 chunks semantically similar to this change in the last hour
  ```

### Edge Cases

* Prometheus unreachable: `ObservabilitySignal.services` is empty; no error surfaced to user; a `DEBUG` log is written
* `service_map` has no entry for a changed file's path: that file's changes contribute no service signals
* Multiple services map to the same changed file: all matched services are queried; the worst signal governs risk elevation
* Log files do not exist at configured paths: silently skipped
* Log file is actively written during scan (race condition): use non-exclusive read; tolerate partial last line
* Embedding model unavailable: log scanning falls back to keyword matching (grep for `ERROR`, `FATAL`, `panic`, `exception` in log chunks near the log_lookback window); anomaly count is still populated
* Risk elevation from observability would push risk to a tier above `High`: `High` is the ceiling; the reason string is still appended to `risk_reasons`
* Large `service_map` with many services: queries are parallelized (up to 8 concurrent); total timeout is still 5 seconds wall-clock

### Acceptance Criteria

* `changeguard impact` on a repo with a configured service that has elevated Prometheus error rate produces `risk_level: High` when it would otherwise be `Low`
* `observability.risk_elevation` reason appears in `risk_reasons` of the packet
* With `prometheus_url = ""`, the observability section is `null` in the packet and `impact` completes normally
* Log anomaly count is non-zero when log files contain lines semantically similar to the current diff
* Total time added to `changeguard impact` by observability fetching does not exceed 6 seconds (5s Prometheus timeout + 1s buffer)

### Verification Gate

* Unit tests: `observability_signal.rs` â€” risk elevation logic, threshold comparison, tier capping
* Unit tests: PromQL query URL construction and response parsing
* Unit tests: log chunk embedding and similarity threshold
* Integration test: mock Prometheus HTTP server returning elevated error rates, assert risk elevation in packet
* CI: passes with all observability config empty

---

## Phase M6: OpenAPI Contract Indexing

### Objective

Parse OpenAPI 3.x and Swagger 2.x specifications, embed each endpoint's description, and match changed source files to semantically related endpoints. Affected contracts appear in the impact packet to flag public API risk.

### Deliverables

* `src/contracts/parser.rs` â€” YAML/JSON spec parser producing a flat list of `ApiEndpoint` structs
* `src/contracts/index.rs` â€” embed and store endpoints in `api_endpoints` + `embeddings` tables
* `src/contracts/matcher.rs` â€” match changed file embedding to endpoint embeddings
* `changeguard index --contracts` flag (or as part of standard `changeguard index`)
* Impact enrichment: `affected_contracts` field populated in `ImpactPacket`
* Human output: `changeguard impact` prints a "Affected API Contracts" table when entries are present

### Functional Requirements

**Parser:**
* Support OpenAPI 3.x (YAML and JSON) and Swagger 2.x (JSON)
* For each path + method combination, extract: `method`, `path`, `summary`, `description`, `tags`, `operationId`
* Concatenate `summary + " " + description + " " + tags.join(" ")` as the text to embed
* If both `summary` and `description` are absent, use `{method} {path}` as the embed text
* Skip endpoints where the embed text is fewer than 10 characters
* Parse failure for a single file: log `WARN` with the parse error and continue; do not abort

**Incremental indexing:**
* Hash the concatenated embed text with `blake3`; skip re-embedding if hash matches stored `content_hash`
* Remove rows from `api_endpoints` for spec paths that no longer exist in `contracts.spec_paths`

**Matching:**
* On `execute_impact()`, for each changed file:
  * Retrieve the file's embedding from `embeddings` where `entity_type = 'file'`
  * If no embedding exists for the file (not yet indexed), skip contract matching for that file
  * Compute cosine similarity between file embedding and all endpoint embeddings
  * Collect endpoints with similarity > 0.5 (configurable threshold)
* Deduplicate across changed files: if the same endpoint is matched by multiple changed files, keep the highest similarity
* Sort by similarity descending; include up to 10 endpoints in `affected_contracts`

**Output table:**
```
Affected API Contracts
 Method  Path               Spec                  Similarity
 POST    /v1/payments       api/openapi.yaml       0.84
 GET     /v1/payments/{id}  api/openapi.yaml       0.71
```

**Risk elevation:**
* If any `affected_contracts` entry has similarity > 0.75 and the changed file contains a public symbol change (from existing `analysis_status`):
  * Add reason: `"Public contract potentially affected: {method} {path}"`
  * This does not automatically elevate the risk tier; it adds a reason that informs the risk score

### Edge Cases

* No spec files configured: `contracts.spec_paths = []`; `affected_contracts` is empty; no error
* Spec file is binary or malformed YAML: log `WARN`, skip file, continue
* Spec contains `$ref` circular references: parser must detect and break cycles; depth limit of 20
* Path templating in OpenAPI (`/users/{id}`): embed the template path as-is, not with example values
* Very large spec (1000+ endpoints): indexing is bounded by embedding batch size (32/request); will take multiple HTTP calls; acceptable latency for an index operation
* File embedding for a changed file is missing (file was never run through `changeguard index`): skip contract matching for that file and log `DEBUG`
* Similarity threshold produces too many matches (>50): cap at 10 after sort; threshold can be raised in config if needed

### Acceptance Criteria

* `changeguard index --contracts` completes without error on a repo with a valid OpenAPI 3.x spec
* `changeguard impact` on a file that implements a matched endpoint includes at least one `affected_contracts` entry with similarity > 0.5
* Re-running `changeguard index --contracts` without spec changes produces zero new embedding writes
* With no spec files configured, `impact` output is unchanged from current behavior
* Contract matching adds fewer than 200ms to `changeguard impact` wall-clock time

### Verification Gate

* Unit tests: `contract_matching.rs` â€” endpoint embedding text construction, similarity threshold, deduplication
* Unit tests: YAML and JSON parser with OpenAPI 3.x and Swagger 2.x fixtures
* Unit tests: circular `$ref` detection
* Integration test: index a fixture spec, run impact on a matching source file, assert `affected_contracts` is populated
* CI: passes with `spec_paths = []`

---

## 10. Milestones

### Milestone M-Alpha â€” Embedding Foundation

Complete:

* Phase M1 (infrastructure, no user-visible features)

Validation: `changeguard doctor` reports local model status. `changeguard index` runs without error. No regressions in existing tests.

### Milestone M-Beta â€” Architectural Intelligence

Complete:

* Phase M2 (document intelligence)
* Phase M3 (local model ask backend)

Validation: `changeguard ask --backend local` works. `changeguard impact` shows `relevant_decisions` for repos with `docs/` or `adr/`. No quota dependency for `ask` in projects with local model configured.

### Milestone M-Gamma â€” Predictive Workflow

Complete:

* Phase M4 (semantic test prediction)

Validation: `changeguard verify --explain` shows rationale. After 10+ verify runs, semantic score meaningfully influences test ordering on familiar change patterns.

### Milestone M-Delta â€” Full Coverage

Complete:

* Phase M5 (metrics and log observability)
* Phase M6 (OpenAPI contract indexing)

Validation: Risk score is elevated when Prometheus signals are above threshold. `affected_contracts` appears in impact packet for repos with OpenAPI specs. All four coverage dimensions have at least one active signal.

---

## 11. New Dependency Additions

The following crates are added for this expansion. All must pass `cargo audit` and `cargo deny check`.

```toml
# Already added in previous work:
ureq = { version = "2", features = ["json"] }

# New additions for this expansion:

# YAML parsing for OpenAPI specs
serde_yaml = "0.9"

# Token estimation (lightweight, no tokenizer model needed)
# Use a simple character/4 heuristic in budget.rs â€” no new crate required.

# Markdown parsing for doc chunking (already present)
# pulldown-cmark = "0.13"  â† already in Cargo.toml

# HTTP mock server for integration tests (dev-dependency only)
[dev-dependencies]
httpmock = "0.7"
```

No new native or FFI dependencies are required. `sqlite-vec` is explicitly deferred to a future optimization phase â€” the BLOB + Rust cosine approach is sufficient for corpora under 100k embeddings (the expected scale for a single repo).

---

## 12. Testing Strategy

### Unit Tests

Use for:

* Cosine similarity computation
* Token budget enforcement
* Document chunker heading/paragraph splitting
* Context assembly order and truncation
* Backend routing logic (local/gemini selection)
* Semantic score blending formula
* PromQL URL construction and response parsing
* OpenAPI endpoint text extraction and hash computation

### Fixture Tests

Use for:

* Chunking of realistic ADR and design document files
* OpenAPI 3.x and Swagger 2.x parsing with real-looking specs
* Log line similarity scoring against known diffs

### Integration Tests (mock server)

Use for:

* Embedding generation â†’ storage â†’ retrieval round-trip
* Doc index: crawl â†’ chunk â†’ embed â†’ retrieve
* Local model context assembly with budget enforcement
* Contract index: parse spec â†’ embed â†’ match to changed file
* Observability: mock Prometheus response â†’ risk elevation

### CI Constraints

* All integration tests that require a live local model server use a mock HTTP server
* CI must pass with all new config sections empty or absent
* No new environment variables required in CI

---

## 13. AI Implementation Protocol

Each AI implementation pass for this plan must follow this discipline:

1. Implement one phase only. Do not implement Phase M2 infrastructure while completing M1.
2. Verify that `local_model.base_url = ""` leaves all existing tests passing before declaring a phase complete.
3. Every new public function has at least one unit test. Every new SQLite table has at least one migration test.
4. Run `cargo fmt --check`, `cargo clippy --all-targets --all-features`, and `cargo test` before marking a phase done.
5. New config fields must have defaults that reproduce the pre-expansion behavior when the feature is not configured.
6. Do not embed secrets. Before sending any text to the embedding API, run it through the existing sanitizer in `src/gemini/sanitize.rs`.
7. The 38k context window is a hard limit, not a target. Build to use as little of it as necessary.

---

## 14. Final Implementation Warning

The most likely failure mode for this expansion is building the embedding pipeline but never getting retrieval precision high enough to make `relevant_decisions` actually useful. A list of loosely-related ADR excerpts that don't match the current change trains users to ignore the feature.

Success criteria are behavioral, not structural:

* A developer changing an auth middleware file sees the ADR that explains why that middleware was chosen
* A developer touching a payment endpoint sees the OpenAPI contract for that endpoint flagged
* A flaky test that has historically failed on changes like this one is ranked first in verification
* An elevated Prometheus error rate on a service matching the changed code is visible before the change is pushed

If retrieval quality is not meeting these criteria after M-Beta is live, recalibrate the similarity thresholds and reranking configuration before proceeding to M-Gamma and M-Delta. Precision matters more than recall here â€” one highly relevant result is more valuable than ten loosely related ones.

Reliability comes first. Semantic sophistication comes second.

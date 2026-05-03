# Specification: Track M1-2 — Cosine Similarity, Top-K, Budget & Doctor

## Objective
Complete the embedding infrastructure with similarity computation, top-k retrieval, token budget enforcement, and doctor health reporting. After this track, all embedding primitives are available for Phases M2–M6.

## Components

### 1. Cosine Similarity (`src/embed/similarity.rs`)

```rust
pub fn cosine_sim(a: &[f32], b: &[f32]) -> f32
```
- Compute: `dot(a, b) / (norm(a) * norm(b))`
- If either norm is 0.0, return 0.0 (no panic, no NaN)
- Input length mismatch: return 0.0 with a `DEBUG` log

```rust
pub fn top_k(query: &[f32], candidates: &[(String, Vec<f32>)], k: usize) -> Vec<(String, f32)>
```
- Score each candidate with `cosine_sim(query, candidate_vec)`
- Sort descending by score
- Return at most `k` entries; if `candidates.len() < k`, return all

### 2. Candidate Loading (`src/embed/storage.rs` addition)

```rust
pub fn load_candidates(
    conn: &Connection,
    entity_type: &str,
    model_name: &str,
) -> Result<Vec<(String, Vec<f32>)>>
```
- SELECT `entity_id`, `vector` WHERE `entity_type = ?` AND `model_name = ?`
- Deserialize BLOB as little-endian `f32` array
- Return `Vec<(entity_id, vec)>`

### 3. Token Budget (`src/embed/budget.rs`)

```rust
pub fn token_estimate(text: &str) -> usize
```
- Approximate: `text.len() / 4`
- Always returns at least 1 for non-empty text

```rust
pub fn enforce_budget<'a>(
    parts: &[&'a str],
    budget: usize,
) -> (Vec<&'a str>, bool)
```
- Greedy: include parts from front until adding the next would exceed budget
- A single part that alone exceeds the budget is still included (never splits within a part)
- Returns `(included_parts, was_truncated)`

### 4. `embed_and_store` Convenience (`src/embed/mod.rs`)

```rust
pub fn embed_and_store(
    config: &LocalModelConfig,
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    text: &str,
) -> Result<bool>
```
- If `config.base_url` is empty: return `Ok(false)` immediately
- Compute `content_hash`; if stored hash matches, return `Ok(false)` (skip)
- Call `embed_batch` for the single text; call `upsert_embedding`
- Return `Ok(true)` on successful new/updated store

```rust
pub fn embed_long_text(
    config: &LocalModelConfig,
    text: &str,
) -> Result<Vec<f32>>
```
- If text fits within model max input tokens: delegates to `embed_batch` for a single text, returns the single vector
- If text exceeds max input: splits into overlapping chunks (max_input_tokens each, 64-token overlap), calls `embed_batch` for all chunks, mean-pools the resulting vectors into one, returns the pooled vector
- Used by `embed_and_store` internally so callers never deal with chunking

### 5. Doctor Extension

New `LocalModelStatus` enum:
```rust
pub enum LocalModelStatus {
    NotConfigured,
    NotReachable { url: String, error: String },
    Found { url: String, model: String },
}
```

`check_local_model(config: &LocalModelConfig) -> LocalModelStatus`:
- If `base_url` is empty: `NotConfigured`
- Send `embed_batch` with `["ping"]` and 5s timeout
- 200 OK with valid response: `Found`
- Any error: `NotReachable`

Doctor output row:
```
Local Model:       Found (http://localhost:8080, text-embedding-nomic-embed-text)
```
or:
```
Local Model:       Not configured
```

## Test Specifications

| Test | Assertion |
|---|---|
| `cosine_sim` identical vectors | Returns 1.0 |
| `cosine_sim` orthogonal vectors | Returns 0.0 |
| `cosine_sim` zero vector | Returns 0.0, no panic |
| `cosine_sim` length mismatch | Returns 0.0, no panic |
| `top_k` 5 candidates, k=3 | Returns 3 results in descending order |
| `top_k` k > candidates.len() | Returns all candidates |
| `token_estimate` non-empty | Returns positive integer |
| `enforce_budget` all fit | Returns all parts, `was_truncated = false` |
| `enforce_budget` overflow | Returns subset, `was_truncated = true` |
| `enforce_budget` single over-budget part | Returns that part, `was_truncated = false` |
| `embed_and_store` empty base_url | Returns `Ok(false)`, no HTTP call |
| `embed_and_store` same text twice | Second call returns `Ok(false)` |
| `load_candidates` 3 stored | Returns 3 with correct entity_ids |
| `check_local_model` empty base_url | Returns `NotConfigured` |
| `check_local_model` mock 200 | Returns `Found` |
| `check_local_model` unreachable | Returns `NotReachable` |

## Constraints & Guidelines

- **TDD**: All tests written before implementation.
- **No panics**: Division-by-zero and length mismatches handled as documented above.
- **Mock tests**: Use `httpmock` for all tests that exercise `check_local_model`.
- **No floating-point surprises**: `cosine_sim` must return values in `[-1.0, 1.0]`; add an `assert` in debug builds.
- **CI safety**: All tests pass with `local_model.base_url = ""` — no network calls in CI.
- **Text length handling**: Embedding text exceeding the model's max input tokens must be chunked (overlapping), each chunk embedded separately, then mean-pooled into a single vector. This preserves semantic information from the entire text rather than silently discarding the tail.

## Hardening Additions (not in original plan)

| Addition | Reason |
|---|---|
| Chunking + mean-pool for texts exceeding model input limit | Truncation would silently discard information from the tail of long texts. Chunking preserves all semantic content equally. The doc chunker (M2-1) already produces small chunks, but `embed_and_store` may receive unbounded text from other callers. |
| Dedicated `embed_long_text()` function: split → embed batch → mean-pool | Encapsulates the chunking logic so callers don't need to worry about token limits. Returns a single pooled embedding vector. |

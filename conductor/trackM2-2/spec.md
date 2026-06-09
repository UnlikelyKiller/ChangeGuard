# Specification: Track M2-2 — Retrieval, Reranking & Impact Enrichment

## Objective
Wire the indexed document chunks into the impact analysis pipeline: retrieve semantically relevant ADR/design doc sections for each change, optionally rerank them, and attach them to the `ImpactPacket` as `relevant_decisions`. Inject them into `ask` context.

## Components

### 1. `RetrievedChunk` Type (`src/retrieval/query.rs`)

```rust
pub struct RetrievedChunk {
    pub entity_id: String,
    pub similarity: f32,
    pub content: String,
    pub heading: Option<String>,
    pub file_path: String,
}
```

### 2. Retrieval Query (`src/retrieval/query.rs`)

**`retrieve_top_k(conn, query_vec, entity_type, model_name, k) -> Result<Vec<RetrievedChunk>>`**

1. Call `load_candidates(conn, entity_type, model_name)` → `Vec<(entity_id, vec)>`
2. Score each with `cosine_sim(query_vec, vec)`
3. Sort descending; take top `k * 3` (over-fetch for reranker)
4. For `entity_type = "doc_chunk"`: resolve `content`, `heading`, `file_path` from `doc_chunks` by `entity_id`
5. Return

### 3. Reranking (`src/retrieval/rerank.rs`)

**`rerank(base_url, model, query, chunks, timeout_secs) -> Result<Vec<RetrievedChunk>>`**

POST to `{base_url}/v1/rerank`:
```json
{
  "model": "<rerank_model>",
  "query": "<query_text>",
  "documents": ["<chunk1.content>", "<chunk2.content>", ...]
}
```
Expected response:
```json
{"results": [{"index": 0, "relevance_score": 0.91}, ...]}
```
- Assign `relevance_score` to `chunk.similarity` (overwrite cosine score)
- Sort descending by new score
- On any error (unreachable, non-200, malformed): return original `chunks` unchanged; log `WARN` once

### 4. New `ImpactPacket` Fields (`src/impact/packet.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RelevantDecision {
    pub file_path: PathBuf,
    pub heading: Option<String>,
    pub excerpt: String,         // content.chars().take(200).collect()
    pub similarity: f32,
    pub rerank_score: Option<f32>,
}

// Manual Ord: sort by similarity descending, then file_path ascending
impl Eq for RelevantDecision {}
impl PartialOrd for RelevantDecision { ... }
impl Ord for RelevantDecision { ... }
```

Add to `ImpactPacket`:
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub relevant_decisions: Vec<RelevantDecision>,
```

### 5. Impact Enrichment (`src/commands/impact.rs`)

After all existing enrichment completes in `execute_impact()`:

1. Build query text: `changed_file_paths.join(" ") + " " + changed_symbol_names.join(" ")`
2. Call `embed_and_store` on query text with `entity_type = "impact_query"`, `entity_id = "<timestamp>"`
3. If embedding succeeded, call `retrieve_top_k` for `entity_type = "doc_chunk"`, `k = config.docs.retrieval_top_k * 3`
4. Call `rerank` on candidates; take final `config.docs.retrieval_top_k`
5. Map to `RelevantDecision`; assign to `packet.relevant_decisions`

Skip entirely (no error) when:
- `config.local_model.base_url` is empty
- `doc_chunks` table has zero rows (checked via `SELECT COUNT(*) FROM doc_chunks`)

### 6. Determinism Contract Updates (`src/impact/packet.rs`)

**`Finalize` additions:**
- Sort `relevant_decisions` in `ImpactPacket::finalize()` by descending similarity

**`Truncate` additions:**
- Clear `relevant_decisions` in `ImpactPacket::truncate_for_context()` Phase 3 (alongside `temporal_couplings`, `structural_couplings`, etc.) to free context budget for oversized packets

### 7. Ask Context Injection (`src/commands/ask.rs`)

`format_relevant_decisions(decisions: &[RelevantDecision]) -> String`:
```
## Relevant Architecture Documents
### {heading} ({file_path})
{excerpt}
---
```
One block per decision. Empty string if `decisions` is empty.

In `execute_ask()`, after loading packet and before building `user_prompt`:
1. Format decisions block
2. Prepend to `user_prompt`
3. Run `enforce_budget` on `[decisions_block, user_prompt_rest, ...]` against `config.local_model.context_window`
4. Log `WARN` if trimming occurred

## Test Specifications

| Test | Assertion |
|---|---|
| `retrieve_top_k` 5 candidates, k=3 | Returns 3 in descending cosine order |
| `retrieve_top_k` empty DB | Returns empty vec |
| `rerank` mock server reverses order | Output in reversed order |
| `rerank` server unreachable | Returns original order |
| `RelevantDecision` serialization | Round-trips correctly |
| `ImpactPacket` empty `relevant_decisions` | Field absent in JSON |
| Impact enrichment — seeded fixture | `relevant_decisions` non-empty |
| Impact enrichment — `base_url` empty | Completes, `relevant_decisions` empty |
| `format_relevant_decisions` 2 items | Produces expected markdown |
| Budget enforcement — overflow | Decisions trimmed from end |

## Constraints & Guidelines

- **TDD**: All tests written before implementations.
- **No blocking on hot path**: If embedding or reranking takes >3s, it is acceptable — `impact` is not interactive. Still respect `timeout_secs`.
- **Excerpt safety**: `excerpt` must never include raw secrets. Run through existing sanitizer before storing.
- **Reranker is optional**: With reranker absent, cosine-similarity ordering is used — this is the documented degradation path, not a bug.
- **Test isolation**: Use `tempfile` for all SQLite and mock HTTP server per test.

## Hardening Additions (not in original plan)

| Addition | Reason |
|---|---|
| `RelevantDecision` implements `Eq + Ord` (sort by similarity descending) | Required by `ImpactPacket::finalize()` determinism contract. All Vec fields must be sorted. |
| `relevant_decisions` cleared in `truncate_for_context()` Phase 3 | Existing 5-phase truncation pattern must include new large fields to honor the 38k context budget. |
| `relevant_decisions` sorted in `finalize()` | Deterministic JSON output for byte-identical impact reports. |

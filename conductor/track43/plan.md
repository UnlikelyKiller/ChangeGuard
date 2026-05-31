# Implementation Plan - Track 43: Smart Context Pruning for Local LLM

## Goal
Make `changeguard ask --backend local` viable on large repositories by replacing naive context truncation with semantic relevance-based pruning.

## Proposed Changes

### 1. Pruner Foundation [src/local_model/pruner.rs] [NEW]
- Define `RankedChunk { content: String, source: String, score: f32 }`.
- Define `PrunedPacket<'a>` as a zero-copy subset view of `ImpactPacket`:
  ```rust
  pub struct PrunedPacket<'a> {
      pub risk_level: &'a RiskLevel,
      pub changes: &'a [ChangedFile],
      pub temporal_couplings: &'a [TemporalCoupling],
      pub decisions: &'a [RelevantDecision],
  }
  ```
- Implement `query_relevant_chunks(query: &str, changed_files: &[PathBuf], storage: &StorageManager, top_k: usize) -> Result<Vec<RankedChunk>>`.
  - Embed the query using the existing `embed_and_store` path (or reuse if already embedded).
  - Query `doc_chunks` and `project_symbols` tables via cosine similarity.
  - **Deduplication pass**: Remove chunks with >`chunk_dedup_threshold` (default 0.95) semantic overlap before injection.
  - Return top-K chunks sorted by similarity.
- Implement `prune_impact_packet(packet: &ImpactPacket, mode: GeminiMode) -> PrunedPacket`.
  - For `ReviewPatch`: keep diff, changed files, risk level; drop observability, contracts, decisions.
  - For `Analyze`: keep risk, changed files, temporal couplings, decisions; drop observability unless risk is High.
  - For `Narrative`: keep risk, hotspots, decisions; trim contracts.
- **Budget allocation framework**:
  - Reserve 10% of `context_window` for the model's response generation.
  - Reserve 5% as safety headroom.
  - Allocate remaining 85% to context: system prompt (fixed) → user query + pruned packet (high priority) → top-ranked chunks.

### 2. Context Assembly Update [src/local_model/context.rs]
- Change `assemble_context` signature to accept `relevant_chunks: &[RankedChunk]`.
- Budget allocation strategy:
  1. System prompt (fixed)
  2. User query + pruned impact packet (high priority)
  3. Top-ranked chunks until budget exhausted
- Use accurate token counting: prefer `tiktoken-rs` or `tokenizers` if available; fall back to `len() / 4` with a logged warning.
- If budget is exceeded after step 2, drop low-ranked chunks and warn.
- If still exceeded, truncate the pruned packet fields in priority order (contracts first, then decisions, then observability).

### 3. Backend Wiring [src/commands/ask.rs]
- In the `Backend::Local` branch (currently passes `&[]` for context chunks):
  - After loading the impact packet, call `prune_impact_packet`.
  - Call `query_relevant_chunks` with the user's query and changed files from the packet.
  - Pass ranked chunks to `assemble_context`.
- Add `LocalModelConfig` tunables: `chunk_top_k`, `chunk_min_similarity`, `chunk_dedup_threshold`.
- **Graceful degradation paths**:
  - No indexed docs → proceed with pruned impact packet only.
  - Embedding server unavailable → fallback to keyword matching or skip chunks entirely.
  - No impact packet → warn and send only system prompt + user query.

### 4. Budget Enforcement Tests
- `test_pruner_respects_token_budget`: Mock 100 chunks of 1k tokens each, 10k budget → only top 10 included.
- `test_pruner_prioritizes_query_relevance`: Ensure chunks with higher similarity scores are included before lower ones.
- `test_prune_impact_packet_review_patch_drops_observability`: Verify mode-specific pruning.
- `test_assemble_context_warns_on_trim`: Assert warning is logged when chunks are dropped.
- `test_pruner_deduplicates_near_duplicates`: Two chunks with 0.99 overlap → only one included.
- `test_pruner_graceful_degradation_no_embeddings`: Embedding server down → falls back gracefully.
- **Stress test**: 1000 fake chunks of 500 tokens each, 1000-token budget → assert exactly 1 chunk fits after system prompt + user query.

### 5. Integration Test
- Mock local model server (using `wiremock` or `mockito`) that receives a request under the budget.
- End-to-end `execute_ask` with local backend on a repo with indexed docs.

## Verification Plan

### Automated Tests
- `cargo test` in `src/local_model/`.
- `cargo test --workspace`.

### Manual Verification
- Run `changeguard ask --backend local "explain auth changes"` on ChangeGuard itself with a dirty auth file. Verify response is under token limit and relevant.

## Definition of Done (DoD)
- [ ] **Pruner Module**: `src/local_model/pruner.rs` exists with `query_relevant_chunks` and `prune_impact_packet`.
- [ ] **Zero-Copy PrunedPacket**: `PrunedPacket` is a lifetime-bound view, not a clone.
- [ ] **Budget Guarantee**: Local backend never exceeds 85% usable context tokens (10% response reserve, 5% safety).
- [ ] **Deduplication**: Near-duplicate chunks (>0.95 overlap) are removed before injection.
- [ ] **Relevance Ranking**: Unit tests prove that higher-similarity chunks are preferred.
- [ ] **Mode-Aware Pruning**: Different `GeminiMode` values produce different field subsets.
- [ ] **Graceful Degradation**: Works without embeddings, without impact packet, and with empty doc index.
- [ ] **Config Tunables**: `chunk_top_k`, `chunk_min_similarity`, `chunk_dedup_threshold` in `LocalModelConfig`.
- [ ] **Stress Test**: Budget enforcement holds under 1000-chunk load.
- [ ] **Zero Regression**: Existing `ask` and local model tests pass.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

# Implementation Plan - Track 43: Smart Context Pruning for Local LLM

## Goal
Make `changeguard ask --backend local` viable on large repositories by replacing naive context truncation with semantic relevance-based pruning.

## Proposed Changes

### 1. Pruner Foundation [src/local_model/pruner.rs] [NEW]
- Define `RankedChunk { content: String, source: String, score: f32 }`.
- Implement `query_relevant_chunks(query: &str, changed_files: &[PathBuf], storage: &StorageManager, top_k: usize) -> Result<Vec<RankedChunk>>`.
  - Embed the query using the existing `embed_and_store` path (or reuse if already embedded).
  - Query `doc_chunks` and `project_symbols` tables via cosine similarity.
  - Return top-K chunks sorted by similarity.
- Implement `prune_impact_packet(packet: &ImpactPacket, mode: GeminiMode) -> PrunedPacket`.
  - For `ReviewPatch`: keep diff, changed files, risk level; drop observability, contracts, decisions.
  - For `Analyze`: keep risk, changed files, temporal couplings, decisions; drop observability unless risk is High.
  - For `Narrative`: keep risk, hotspots, decisions; trim contracts.

### 2. Context Assembly Update [src/local_model/context.rs]
- Change `assemble_context` signature to accept `relevant_chunks: &[RankedChunk]`.
- Budget allocation strategy:
  1. System prompt (fixed)
  2. User query + pruned impact packet (high priority)
  3. Top-ranked chunks until budget exhausted
- If budget is exceeded after step 2, drop low-ranked chunks and warn.
- If still exceeded, truncate the pruned packet fields in priority order (contracts first, then decisions, then observability).

### 3. Backend Wiring [src/commands/ask.rs]
- In the `Backend::Local` branch:
  - After loading the impact packet, call `prune_impact_packet`.
  - Call `query_relevant_chunks` with the user's query and changed files from the packet.
  - Pass ranked chunks to `assemble_context`.

### 4. Budget Enforcement Tests
- `test_pruner_respects_token_budget`: Mock 100 chunks of 1k tokens each, 10k budget → only top 10 included.
- `test_pruner_prioritizes_query_relevance`: Ensure chunks with higher similarity scores are included before lower ones.
- `test_prune_impact_packet_review_patch_drops_observability`: Verify mode-specific pruning.
- `test_assemble_context_warns_on_trim`: Assert warning is logged when chunks are dropped.

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
- [ ] **Budget Guarantee**: Local backend never sends more than `context_window` tokens (10% headroom reserved).
- [ ] **Relevance Ranking**: Unit tests prove that higher-similarity chunks are preferred.
- [ ] **Mode-Aware Pruning**: Different `GeminiMode` values produce different field subsets.
- [ ] **Zero Regression**: Existing `ask` and local model tests pass.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

# Track 43: Smart Context Pruning for Local LLM

## Overview
When `changeguard ask --backend local` is invoked, the current implementation sends the entire impact packet, observability signals, affected contracts, and relevant decisions to the local model. On large repositories this payload can exceed ~80k tokens, while typical local models accept ~39k. The existing `assemble_context` only performs a naive head truncation: it includes context chunks in order until the token budget is exhausted. There is no relevance-based selection, making the local backend impractical for non-trivial repositories.

## Objectives
- Implement a relevance-aware context-pruning pipeline that selects only file chunks and documentation snippets semantically related to the user's query and the current change set.
- Guarantee that the local backend never exceeds the configured token budget.
- Preserve the ability to see full context when the query is broad (e.g., "analyze everything").

## Architecture
- `src/local_model/pruner.rs` [NEW] — Core pruning engine.
  - `rank_chunks(query_embedding, diff_summary, chunk_embeddings) -> Vec<RankedChunk>`
  - `prune_impact_packet(packet, mode) -> PrunedPacket` — zero-copy subset view dropping observability/contracts/decisions that are irrelevant to the query mode.
- `src/local_model/context.rs` — Update `assemble_context` to accept `&[RankedChunk]` instead of raw `&[&str]`, and sort by relevance before budget enforcement.
- `src/retrieval/query.rs` — Reuse existing `query_embeddings` and cosine-similarity logic; add deduplication pass for chunks with >0.95 semantic overlap.
- `src/commands/ask.rs` — Wire pruner into the local backend path; query `doc_chunks`/`project_symbols`, rank, and pass to `assemble_context`.
- `src/config/model.rs` — Add `LocalModelConfig` tunables: `chunk_top_k`, `chunk_min_similarity`, `chunk_dedup_threshold`.

## Success Criteria
- Local backend token count never exceeds `config.local_model.context_window` (with 10% headroom reserved for response generation; 85% usable for context).
- On a repo with >100 indexed documents, a targeted query (e.g., "how does auth work?") includes only auth-related chunks, not the full repo.
- A broad query (e.g., "analyze impact") gracefully degrades to the most important fields (risk level, changed files) when the full packet doesn't fit.
- Graceful degradation: if embedding server is unavailable, keyword fallback or no chunks; if no impact packet, warn and send system prompt + user query only.
- All existing `ask` tests pass.
- New unit tests for `pruner.rs`, updated `assemble_context` tests, and a budget stress test (1000 chunks vs 1000-token budget).

## Testing Strategy
- **Red commit**: Add tests that simulate an 80k-token repo context against a 40k-token budget. Assert that naive assembly fails (exceeds budget) and pruned assembly succeeds.
- **Green commit**: Implement pruner. Verify budget compliance and relevance ranking.

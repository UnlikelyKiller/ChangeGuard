# Track K15: Semantic Search Readiness and Fallbacks

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`search --semantic` can spend tens of seconds on model/vector work and still return no relevant snippets for common codebase queries. The command technically succeeds, but the user receives little explanation about readiness, freshness, or why no semantic result was produced.

## Objective
Make semantic search predictable by checking readiness before expensive work, explaining empty results, and offering a useful lexical fallback when semantic search has no signal.

## Scope
- Validate semantic index freshness, vector count, embedding dimensions, embedding model name, and local embedding endpoint readiness before query execution.
- Support split local-model deployments where embeddings are served by a CPU endpoint such as Nomic Embed v1.5 on `127.0.0.1:8083`, independent from the chat model endpoint.
- Add clear empty-result diagnostics.
- Consider fallback to BM25 or blended lexical results when semantic retrieval returns nothing.
- Preserve `--json` behavior with structured readiness and fallback metadata.

## Non-Goals
- Do not silently rebuild semantic indexes unless the user explicitly requests indexing or auto-index behavior.
- Do not send semantic embedding requests to the generation endpoint.
- Do not hide readiness problems by returning unrelated lexical results without explaining the fallback.

## Implementation Notes
- Store or expose enough semantic index metadata to compare configured embedding model and dimensions with indexed vectors.
- When fallback is used, output should identify semantic status and fallback source separately.
- Nomic Embed v1.5 should be treated as a supported CPU embedding deployment for acceptance testing.

## Success Criteria
- [ ] Semantic search reports readiness problems before long waits.
- [ ] Empty semantic results explain whether the cause is no indexed snippets, embedding endpoint unavailability, model mismatch, dimension mismatch, or low similarity.
- [ ] Switching from `bge-m3` dimensions to Nomic Embed v1.5 dimensions produces a clear rebuild-required diagnostic instead of degraded retrieval.
- [ ] A fallback or suggestion returns useful next steps for common lexical queries.
- [ ] Tests cover unavailable model, empty vector store, stale semantic index, and no-result cases.
- [ ] CI gate passes.

## Definition of Done
- [ ] `search --semantic` performs a readiness check before expensive retrieval.
- [ ] `search --semantic --json` includes structured readiness, no-result reason, and fallback metadata.
- [ ] A bge-to-Nomic model/dimension change produces a rebuild-required diagnostic with the exact indexing command.
- [ ] Semantic search against LLM2 Nomic Embed v1.5 on `127.0.0.1:8083` succeeds after rebuilding the semantic index.
- [ ] Tests cover unavailable endpoint, empty vector store, stale index, model mismatch, dimension mismatch, no-result, and fallback cases.
- [ ] `changeguard verify` passes.
- [ ] `cargo install --path . --force` succeeds and installed-binary semantic search smoke checks pass.

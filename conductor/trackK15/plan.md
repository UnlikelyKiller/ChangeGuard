# Track K15 Plan: Semantic Search Readiness and Fallbacks

## Phase 1: Readiness Model
- [ ] Define semantic readiness fields for index freshness, vector count, dimensions, model name, and endpoint availability.
- [ ] Add tests for unavailable model and empty semantic index states.
- [ ] Add tests for embedding endpoint/model/dimension mismatch after switching providers.
- [ ] Identify where readiness belongs in human and JSON output.
- [ ] Identify how semantic index metadata records the embedding model and dimensions.

## Phase 2: Query Behavior
- [ ] Run readiness checks before expensive semantic retrieval.
- [ ] Add structured no-result reasons.
- [ ] Emit a rebuild-required diagnostic when configured embedding dimensions differ from stored semantic vectors.
- [ ] Add BM25 fallback or explicit suggested command when semantic results are empty, with fallback clearly labeled.

## Phase 3: Verification
- [ ] Run semantic search with local model unavailable.
- [ ] Run semantic search against LLM2 Nomic Embed v1.5 on `127.0.0.1:8083`.
- [ ] Run semantic search with stale or empty semantic index.
- [ ] Run semantic search where BM25 fallback should produce results.
- [ ] Validate `search --semantic --json` with a JSON parser.
- [ ] Run `cargo install --path . --force` and repeat installed-binary semantic checks.
- [ ] Run full CI gate.

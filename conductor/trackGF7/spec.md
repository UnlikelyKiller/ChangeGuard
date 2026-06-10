# Track GF7: Index Command Mode Extraction

## Objective

Extract `src/commands/index.rs` mode handlers so `execute_index` becomes readable, testable command orchestration instead of a large mixed-mode function. Correction (verified 2026-06-09): four handlers are **already extracted** as private functions (`execute_docs_index`, `execute_semantic_index`, `execute_scip_index`, `execute_semantic_dry_run`); the remaining inline modes in `execute_index` are check, incremental/full, analyze-graph, export-docs, contracts wiring, and the `--fast` Gemini extraction path. The real work is finishing the extraction, normalizing shared options, and capturing the mode-combination semantics that are currently implicit in early-return ordering.

## Evidence

- User analysis ranks `src/commands/index.rs` as refactor need 6/10.
- `changeguard hotspots trend` lists `src/commands/index.rs` as a current hotspot.
- `changeguard hotspots explain src/commands/index.rs` reports temporal coupling above 70% with `src/config/validate.rs` at 0.80.
- This track should follow GF3 and GF6 so command mode handlers can call stable lower-level indexing APIs.

## Scope

Required mode boundaries:

- `mode_docs`: docs crawling/chunking/indexing.
- `mode_contracts`: OpenAPI/contract indexing.
- `mode_analyze_graph`: native graph build and graph freshness output, including the `--fast` Gemini semantic-extraction path.
- `mode_semantic`: embedding-backed indexing and unavailable-model handling.
- `mode_semantic_dry_run`: resolved-settings printout (`--semantic-dry-run`, with optional JSON output path).
- `mode_incremental`: incremental index refresh.
- `mode_check`: index health and stale-index reporting, including `--strict` and `--json` interplay.
- `mode_scip`: SCIP ingestion.
- `mode_export_docs`: passive documentation export (`--export-docs`, `--doc-type`).
- `options`: shared option normalization and validation, including `--concurrency` (rayon thread count) resolution.
- A documented **mode-combination matrix**. The current dispatch order encodes non-obvious semantics that must be characterized before extraction: `--semantic-dry-run` preempts everything; `--scip` early-returns next; `--semantic` early-returns only when `--analyze-graph` is absent (so `--semantic --analyze-graph` falls through to the graph path); `--docs --analyze-graph` runs both. Extraction must preserve this precedence exactly.
- Preserve stdout/stderr behavior, progress messages, JSON/script safety, and side effects.

## Non-Goals

- Do not change the `changeguard index` CLI surface.
- Do not redesign `IndexOrchestrator`.
- Do not make semantic model availability mandatory.

## Implementation Notes

- Move mode bodies into functions first; create modules only when the functions are cohesive and stable.
- Centralize option validation so modes do not drift.
- Preserve existing tests and add missing mode-level smokes.
- Keep human progress off JSON stdout.

## Verification Strategy

Targeted:

- `cargo test commands::index`
- A new `tests/integration/cli_index.rs` suite — verified 2026-06-09: **no `cli_index` integration module exists today** (the closest coverage is `scip_integration`, `incremental_graph_consistency`, and `watch_graph_sync`). Creating it is in scope for this track; "run existing index command tests" is not sufficient.
- CLI smokes for every `changeguard index` mode available without network.

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `execute_index` delegates to named mode handlers.
- Every index mode has focused test or smoke coverage.
- Graph/search/index state behavior is unchanged.
- JSON/script-safe output remains clean.
- Final verification and reinstall pass.

## Risks

- Mode extraction can accidentally change ordering of side effects.
- Semantic indexing failures can hide if tests only run on machines without model config.
- Progress output can corrupt structured stdout if moved carelessly.

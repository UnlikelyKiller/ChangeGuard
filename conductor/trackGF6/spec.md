# Track GF6: Index Orchestrator Capability Split

## Objective

Split `src/index/orchestrator.rs` into focused indexing capabilities while preserving **`ProjectIndexer`** as the public facade. (Correction, 2026-06-09: the facade struct is `ProjectIndexer`, not `IndexOrchestrator` — earlier drafts and the conductor entry used the wrong name.) The file spans file scanning, tree-sitter parsing, index lifecycle, extraction delegation, centrality, services inference, KG build delegation, and raw SQL row helpers.

## Evidence

- User analysis ranks `src/index/orchestrator.rs` as refactor need 7/10 due to broad orchestration domains in one file and limited focused tests.
- GF3 should extract graph loader phases first so this track can delegate graph work rather than entangle both refactors.
- Indexing feeds search, ask, endpoints, services, observability, security, tests, and dependency surfaces, so behavior compatibility is more important than reducing line count quickly.
- Verified 2026-06-09: SCIP ingestion, semantic indexing, and documentation export are **not** in this file — they are orchestrated from `src/commands/index.rs` (`execute_scip_index`, `execute_semantic_index`, `execute_semantic_dry_run`) and belong to GF7. Do not pull them into this track.

## Scope

Required capability boundaries, aligned to the actual `ProjectIndexer` method surface:

- `discovery`: `discover_files`, `discover_doc_files`, ignore rules, changed-file selection.
- `parsing`: `index_file`, `index_file_with_edges`, tree-sitter setup, language dispatch, syntax failure reporting.
- `lifecycle`: `full_index`, `incremental_index`, `check_status`, and stale-index reporting.
- `extraction`: delegation methods `build_call_graph`, `extract_routes`, `extract_data_models`, `extract_observability`, `extract_test_mappings`, `extract_ci_gates`, `extract_env_schema`, plus the matching `clear_*` methods.
- `topology`: `index_topology`, `classify_entrypoints`, `infer_services`.
- `centrality`: `compute_centrality` and complexity calculations.
- `graph`: `build_kg_native` invocation and graph freshness status (delegating to the GF3 phase functions).
- `rows`: the free SQL row helpers (`insert_file_row`, `upsert_file_row`, `get_file_id_by_path`, `delete_file_index_dependents`, `insert_symbol_row`) into a persistence-side module.
- `docs`: `index_docs` doc-chunk crawling (indexing only — passive doc *export* stays in GF7's command layer).
- Preserve `ProjectIndexer` construction and public methods.

## Non-Goals

- Do not redesign the index state schema.
- Do not remove existing `changeguard index` modes.
- Do not combine this with command-mode extraction; GF7 owns command surface cleanup.

## Implementation Notes

- Extract cohesive helper structs only when they own state or make tests easier.
- Keep path normalization centralized and Windows-safe.
- Use existing parsing fixtures rather than broad new sample repos unless needed.
- Preserve stale-index and auto-index behavior.

## Verification Strategy

Targeted:

- `cargo test index`
- `changeguard index --incremental`
- `changeguard index --analyze-graph`
- `changeguard search "ProjectIndexer" --auto-index`

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `ProjectIndexer` remains the stable facade.
- Capability modules have focused tests or command smokes.
- Incremental and full graph indexing remain idempotent.
- Search index and graph state remain current after indexing.
- Final verification and reinstall pass.

## Risks

- Index freshness can regress without obvious compile failures.
- Path normalization and ignore-rule behavior are easy to break on Windows.
- Semantic indexing may be unavailable locally; tests must degrade gracefully.

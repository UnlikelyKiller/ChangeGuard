# Track GF6 Plan: Index Orchestrator Capability Split

## Phase 0: Baseline and Dependency Check

- [x] Confirm GF3 is complete or explicitly decide which graph-loader boundaries are stable enough to depend on.
- [x] Confirm ledger state with `changeguard ledger status --compact`.
- [x] Start the track transaction: `changeguard ledger start trackGF6 --category REFACTOR --message "ProjectIndexer capability split"`.
- [x] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [x] Run `changeguard index --incremental`.
- [x] Run `changeguard index --analyze-graph`.
- [x] Run `cargo test index`.

Definition of done: Index and graph baselines are known before orchestrator movement.

## Phase 1: Discovery and Parsing

- [x] Extract file discovery helpers.
- [x] Extract ignore and path normalization helpers if they are local to the orchestrator.
- [x] Extract parser/language dispatch helpers.
- [x] Add or update tests for Windows path and ignored-file behavior.
- [x] Run `cargo test index`.

Definition of done: Input selection and parsing boundaries are separated and tested.

## Phase 2: Symbols, References, and Graph

- [x] Extract symbol extraction orchestration.
- [x] Extract reference and route-reference orchestration.
- [x] Extract graph-loading invocation and freshness reporting.
- [x] Run `changeguard index --analyze-graph` and graph surface smokes.

Definition of done: Code intelligence phases are separated and still feed the KG.

## Phase 3: Lifecycle, Topology, Centrality, and Row Helpers

- [x] Extract index lifecycle orchestration (`full_index`, `incremental_index`, `check_status`).
- [x] Extract topology and services orchestration (`index_topology`, `classify_entrypoints`, `infer_services`).
- [x] Extract centrality calculation orchestration.
- [x] Move the free SQL row helpers (`insert_file_row`, `upsert_file_row`, `get_file_id_by_path`, `delete_file_index_dependents`, `insert_symbol_row`) from the orchestrator facade into the sibling `rows` module. Callers are updated to import directly from `crate::index::rows`; both modules remain public in `src/index/mod.rs`.
- [x] Extract doc-chunk crawling (`index_docs`); leave SCIP/semantic/docs-export alone — they belong to GF7.
- [x] Run focused tests or smokes for each capability.

Definition of done: Remaining capabilities are owned by named modules and command-layer concerns stayed out of this track.

## Phase 4: Final Verification

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo nextest run --lib --bins --workspace`.
- [x] Run `cargo nextest run --test integration`.
- [x] Run `changeguard verify`.
- [x] Run `cargo install --path .`.
- [x] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF6" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [x] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.
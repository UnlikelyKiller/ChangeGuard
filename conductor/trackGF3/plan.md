# Track GF3 Plan: Native Graph Loader Phase Extraction

## Phase 0: Baseline and Graph Snapshot

- [x] Confirm ledger state with `changeguard ledger status --compact`.
- [x] Start the track transaction: `changeguard ledger start trackGF3 --category REFACTOR --message "Graph loader phase extraction"`.
- [x] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [x] Run `changeguard hotspots explain src/index/graph_loader.rs`.
- [x] Run `changeguard index --analyze-graph`.
- [x] Capture smoke outputs for `dependencies list`, `security boundaries --json`, `observability coverage --json`, `endpoints --json`, and `services diff`.

Definition of done: Current graph behavior and hotspot/coupling signals are recorded before extraction.

## Phase 1: Context and First Phase Extraction

- [x] Introduce an internal `GraphLoadContext`.
- [x] Extract file-node loading into `phase_files`.
- [x] Add or update tests for path normalization and stale file pruning.
- [x] Note: Phase-level unit tests were deferred. See F5 resolution in `docs/GF-review.md` and the `#[cfg(test)]` module in `graph_loader.rs` for the coverage rationale.
- [x] Run `cargo test index::graph_loader`.

Definition of done: The first phase extraction compiles and preserves file-node behavior.

## Phase 2: Core Code Graph Phases

- [x] Extract symbol-node loading into `phase_symbols`.
- [x] Extract imports/references/calls into `phase_call_edges`.
- [x] Extract route/endpoint loading into `phase_routes`.
- [x] Run graph-related integration tests after each extraction.

Definition of done: Code graph phases are named and test-covered.

## Phase 3: Surface Enrichment Phases

- [x] Extract dependency loading.
- [x] Extract deployment loading.
- [x] Extract environment/config loading.
- [x] Extract observability loading.
- [x] Extract security policy loading and orphan pruning.
- [x] Run affected command smokes after each extraction.

Definition of done: W-surface graph writes are separated and still produce expected rows.

## Phase 4: Idempotence and Final Verification

- [x] Run `changeguard index --analyze-graph` twice.
- [x] Confirm graph surfaces do not duplicate rows after repeated indexing.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo nextest run --lib --bins --workspace`.
- [x] Run `cargo nextest run --test integration`.
- [x] Run `changeguard verify`.
- [x] Run `cargo install --path .`.
- [x] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF3" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [x] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.
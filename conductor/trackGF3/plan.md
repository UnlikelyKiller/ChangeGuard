# Track GF3 Plan: Native Graph Loader Phase Extraction

## Phase 0: Baseline and Graph Snapshot

- [ ] Confirm ledger state with `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF3 --category REFACTOR --message "Graph loader phase extraction"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `changeguard hotspots explain src/index/graph_loader.rs`.
- [ ] Run `changeguard index --analyze-graph`.
- [ ] Capture smoke outputs for `dependencies list`, `security boundaries --json`, `observability coverage --json`, `endpoints --json`, and `services diff`.

Definition of done: Current graph behavior and hotspot/coupling signals are recorded before extraction.

## Phase 1: Context and First Phase Extraction

- [ ] Introduce an internal `GraphLoadContext`.
- [ ] Extract file-node loading into `phase_files`.
- [ ] Add or update tests for path normalization and stale file pruning.
- [ ] Run `cargo test index::graph_loader`.

Definition of done: The first phase extraction compiles and preserves file-node behavior.

## Phase 2: Core Code Graph Phases

- [ ] Extract symbol-node loading into `phase_symbols`.
- [ ] Extract imports/references/calls into `phase_call_edges`.
- [ ] Extract route/endpoint loading into `phase_routes`.
- [ ] Run graph-related integration tests after each extraction.

Definition of done: Code graph phases are named and test-covered.

## Phase 3: Surface Enrichment Phases

- [ ] Extract dependency loading.
- [ ] Extract deployment loading.
- [ ] Extract environment/config loading.
- [ ] Extract observability loading.
- [ ] Extract security policy loading and orphan pruning.
- [ ] Run affected command smokes after each extraction.

Definition of done: W-surface graph writes are separated and still produce expected rows.

## Phase 4: Idempotence and Final Verification

- [ ] Run `changeguard index --analyze-graph` twice.
- [ ] Confirm graph surfaces do not duplicate rows after repeated indexing.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF3" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.

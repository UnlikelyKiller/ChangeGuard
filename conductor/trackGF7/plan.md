# Track GF7 Plan: Index Command Mode Extraction

## Phase 0: Baseline and Mode Matrix

- [ ] Confirm GF3 and GF6 are complete or explicitly scope around unfinished lower-level refactors.
- [ ] Confirm ledger state with `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF7 --category REFACTOR --message "Index command mode extraction"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `changeguard hotspots explain src/commands/index.rs`.
- [ ] List all current `changeguard index --help` modes and flags, including `--fast`, `--export-docs`/`--doc-type`, `--semantic-dry-run`, `--concurrency`, and `--strict`.
- [ ] Write the mode-combination matrix from the current `execute_index` early-return order (dry-run > scip > semantic-without-analyze-graph > docs > graph path) and commit it to the spec or a test comment.
- [ ] Create `tests/integration/cli_index.rs` with characterization tests for the offline modes (none exists today; `cargo test commands::index` and `scip_integration` are the only current coverage).

Definition of done: The implementer has a complete mode matrix, a characterization suite, and baseline behavior.

## Phase 1: Shared Options and First Mode

- [ ] Extract shared option normalization.
- [ ] Extract a low-risk mode handler first, such as `check`.
- [ ] Add or update a test for that mode.
- [ ] Run `cargo check --all-targets --all-features`.

Definition of done: The handler pattern is proven before touching heavier modes.

## Phase 2: Extract Remaining Modes

- [ ] Consolidate the already-extracted handlers (`execute_docs_index`, `execute_semantic_index`, `execute_scip_index`, `execute_semantic_dry_run`) under the shared option-normalization pattern.
- [ ] Extract contracts wiring.
- [ ] Extract analyze-graph mode, including the `--fast` Gemini path.
- [ ] Extract incremental/full mode.
- [ ] Extract export-docs mode (`--export-docs`, `--doc-type`).
- [ ] Run mode-specific tests or smokes after each extraction, asserting the mode-combination matrix still holds.

Definition of done: Each index mode is independently navigable and testable.

## Phase 3: Output and Side-Effect Audit

- [ ] Verify progress output stays on stderr where required.
- [ ] Verify JSON/script-safe output remains parseable.
- [ ] Run index health and stale-index smokes.
- [ ] Run graph/search freshness checks.

Definition of done: Extraction has not changed user-visible contracts or state behavior.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF7" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.

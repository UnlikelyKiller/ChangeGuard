# Track GF9 Plan: Python AST Parser Extraction

## Phase 0: Baseline and Guardrails

- [ ] Confirm ledger state: `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF9 --category REFACTOR --message "Python AST parser extraction by extraction concern"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `cargo test index::languages::python` and record the baseline (all tests pass).
- [ ] Run `cargo check --all-targets --all-features` and confirm clean.
- [ ] Inventory every `pub fn` in `python.rs` and confirm each maps to a destination module in the spec table.
- [ ] Confirm the helper usage audit from the spec still holds: `extract_py_attribute_name` is the only cross-module helper; `extract_py_attribute_object` and `is_in_py_test` are observability-only; `find_py_enclosing_function` is calls-only.

Definition of done: Shared dependencies confirmed; baseline tests recorded; ledger transaction open.

## Phase 1: Directory Scaffold and common.rs

- [ ] Create `src/index/languages/python/` directory. `python.rs` stays in place as the facade — do NOT rename it to `python/mod.rs` (having both `python.rs` and `python/mod.rs` is E0761; the facade-file + directory pattern used by GF8's `dead_code.rs` is the target shape).
- [ ] Create `src/index/languages/python/common.rs` containing `extract_py_attribute_name`, marked `pub(super)`.
- [ ] Add `mod common;` to `python.rs` and replace the inline definition with `use common::extract_py_attribute_name;` (or fully-qualified calls).
- [ ] Run `cargo check --all-targets --all-features`.

Definition of done: Facade + directory pattern compiles with the shared helper extracted.

## Phase 2: Extraction Module Moves

Move one extraction concern at a time. After each, run `cargo check --all-targets --all-features`.

- [ ] Create `src/index/languages/python/symbols.rs`; move `extract_symbols` (make it `pub(super)` in the module).
  - Add `mod symbols; pub use symbols::extract_symbols;` to `python.rs`.
  - Run `cargo check`.
- [ ] Create `src/index/languages/python/routes.rs`; move `extract_routes`, `detect_fastapi_routers`, `detect_flask_objects`, `collect_py_routes`, `extract_py_decorator_path`, `find_py_decorated_function_name`, `extract_flask_method_from_decorator`, `PY_HTTP_METHODS`.
  - Add `mod routes; pub use routes::extract_routes;` to `python.rs`.
  - Run `cargo check`.
- [ ] Create `src/index/languages/python/calls.rs`; move `extract_calls`, `collect_py_call_edges`, `find_py_enclosing_function`.
  - Add `mod calls; pub use calls::extract_calls;` to `python.rs`.
  - Wire `use super::common::extract_py_attribute_name;` inside `calls.rs`.
  - Run `cargo check`.
- [ ] Create `src/index/languages/python/models.rs`; move `extract_data_models`, `collect_py_data_models`.
  - Add `mod models; pub use models::extract_data_models;` to `python.rs`.
  - Run `cargo check`.
- [ ] Create `src/index/languages/python/observability.rs`; move `extract_logging_patterns`, `collect_py_logging_patterns`, `extract_error_handling`, `collect_py_error_handling`, `extract_telemetry_patterns`, `collect_py_telemetry_patterns`, plus the observability-only helpers `extract_py_attribute_object` and `is_in_py_test`.
  - Add `mod observability; pub use observability::{extract_logging_patterns, extract_error_handling, extract_telemetry_patterns};` to `python.rs`.
  - Wire `use super::common::extract_py_attribute_name;` inside `observability.rs`.
  - Run `cargo check`.
- [ ] Verify `python.rs` contains only `mod` declarations and `pub use` re-exports (≤ 30 lines).

Definition of done: All extraction functions reside in their destination modules; `python.rs` is a pure facade; `cargo check` is clean.

## Phase 3: Test Relocation and Coverage

- [ ] Relocate each test from the facade's `#[cfg(test)]` block to the module whose functions it exercises; keep any cross-concern test in the facade.
- [ ] Run `cargo test index::languages::python` and confirm the count matches the Phase 0 baseline.
- [ ] Add at least one round-trip test per new module (parse sample Python input → assert expected output) if no existing test covers it.

Definition of done: Each module has at least one test; test count is at or above baseline; all pass.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit: `changeguard ledger commit <tx-id> --summary "Completed Track GF9: Python AST parser extracted into focused modules" --reason "1,471-line monolith split by extraction concern; mirrors RE5 Rust decomposition"`. If the git pre-commit hook removed the sidecar and `ledger status` still shows 1 pending after the git commit, run `ledger commit` again immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.
- [ ] Mark all tasks `- [x]` in this plan and set Status: Completed in `conductor/conductor.md`.

Definition of done: Full gates pass; installed binary matches source; ledger clean; conductor registry current.

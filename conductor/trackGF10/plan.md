# Track GF10 Plan: TypeScript AST Parser Extraction

## Phase 0: Baseline and Guardrails

- [ ] Confirm ledger state: `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF10 --category REFACTOR --message "TypeScript AST parser extraction by extraction concern"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `cargo test index::languages::typescript` and record the baseline.
- [ ] Run `cargo check --all-targets --all-features` and confirm clean.
- [ ] Re-verify the helper usage audit from the spec with grep: `extract_ts_member_name` and `extract_ts_object_name` are the cross-module helpers; `is_in_ts_test`, `is_in_ts_test_from_line`, `truncate_str` are observability-only.

Definition of done: Shared dependencies confirmed against current code; baseline recorded; ledger open.

## Phase 1: Directory Scaffold and common.rs

- [ ] Create `src/index/languages/typescript/` directory. `typescript.rs` stays as the facade — do NOT rename to `typescript/mod.rs` (E0761 risk; GF8 facade-file pattern is the target shape).
- [ ] Create `src/index/languages/typescript/common.rs` with `extract_ts_member_name` and `extract_ts_object_name`, marked `pub(super)`.
- [ ] Add `mod common;` to `typescript.rs` and rewire the remaining in-facade callers via `use common::{...};`.
- [ ] Run `cargo check --all-targets --all-features`.

Definition of done: Facade + directory compiles with both shared helpers extracted.

## Phase 2: Extraction Module Moves

Move one extraction concern at a time. After each step run `cargo check --all-targets --all-features`.

- [ ] Create `typescript/symbols.rs`; move `extract_symbols`.
  - Add `mod symbols; pub use symbols::extract_symbols;` to `typescript.rs`.
  - Run `cargo check`.
- [ ] Create `typescript/routes.rs`; move `extract_routes`, `detect_fastify`, `collect_ts_routes`, `extract_ts_string_literal`.
  - Add `mod routes; pub use routes::extract_routes;` to `typescript.rs`.
  - Wire `use super::common::{extract_ts_member_name, extract_ts_object_name};` inside `routes.rs`.
  - Run `cargo check`.
- [ ] Create `typescript/calls.rs`; move `extract_calls`, `collect_ts_call_edges`, `find_ts_enclosing_function`.
  - Add `mod calls; pub use calls::extract_calls;` to `typescript.rs`.
  - Wire `use super::common::extract_ts_member_name;` inside `calls.rs`.
  - Run `cargo check`.
- [ ] Create `typescript/models.rs`; move `extract_data_models`, `collect_ts_data_models`.
  - Add `mod models; pub use models::extract_data_models;` to `typescript.rs`.
  - Run `cargo check`.
- [ ] Create `typescript/observability.rs`; move `extract_logging_patterns`, `collect_ts_logging_patterns`, `extract_error_handling`, `collect_ts_error_handling`, `extract_telemetry_patterns`, `collect_ts_telemetry_patterns`, `is_in_ts_test`, `is_in_ts_test_from_line`, `truncate_str`.
  - Add `mod observability; pub use observability::{extract_logging_patterns, extract_error_handling, extract_telemetry_patterns};` to `typescript.rs`.
  - Wire `use super::common::{extract_ts_member_name, extract_ts_object_name};` inside `observability.rs`.
  - Run `cargo check`.
- [ ] Verify `typescript.rs` contains only `mod` declarations and `pub use` re-exports (≤ 30 lines).

Definition of done: All extraction functions in destination modules; facade is pure; clean compile.

## Phase 3: Test Relocation and Coverage

- [ ] Relocate each test from the facade's `#[cfg(test)]` block to the module it exercises; keep cross-concern tests in the facade.
- [ ] Add at least one round-trip test per new module if no existing test covers it.
- [ ] Run `cargo test index::languages::typescript` and confirm the count is at or above the Phase 0 baseline.

Definition of done: All tests pass; each module independently verifiable.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit: `changeguard ledger commit <tx-id> --summary "Completed Track GF10: TypeScript AST parser extracted into focused modules" --reason "1,362-line monolith split by extraction concern; mirrors RE5 and GF9"`. If the git pre-commit hook removed the sidecar and `ledger status` still shows 1 pending after the git commit, run `ledger commit` again immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.
- [ ] Mark all tasks `- [x]` in this plan and set Status: Completed in `conductor/conductor.md`.

Definition of done: Full gates pass; installed binary matches source; ledger clean; conductor registry current.

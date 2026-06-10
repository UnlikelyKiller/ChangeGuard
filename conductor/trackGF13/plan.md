# Track GF13 Plan: Entrypoint Language Detector Split

## Phase 0: Baseline and Guardrails

- [ ] Confirm ledger state: `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF13 --category REFACTOR --message "Entrypoint language detector split by language"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `cargo test index::entrypoint` and record the baseline.
- [ ] Run `cargo check --all-targets --all-features` and confirm clean.
- [ ] Inventory all `pub` functions in `entrypoint.rs` and confirm each maps to a destination module.
- [ ] Identify all test fixtures in the `#[cfg(test)]` block (line 799+); note which language each belongs to. Beware: fixtures are raw-string literals containing column-0 code.

Definition of done: Complete type and function mapping; fixture assignments noted; ledger open.

## Phase 1: Directory Scaffold

- [ ] Create `src/index/entrypoint/` directory. `entrypoint.rs` stays as the facade — do NOT rename to `entrypoint/mod.rs` (E0761 risk; GF8 facade-file pattern is the target shape).
- [ ] No empty stub files — each language module is created in the step that moves its code.

Definition of done: Directory exists; facade untouched; clean compile.

## Phase 2: Language Detector Moves

Move one language detector at a time. After each: `cargo check --all-targets --all-features` and `cargo test index::entrypoint`.

- [ ] Create `entrypoint/rust.rs`; move `detect_rust_entrypoints` (make `pub(super)`), `is_rust_handler_attr`, `parse_rust_attributes`, `extract_rust_attr_path`.
  - Add `mod rust; pub use rust::detect_rust_entrypoints;` to `entrypoint.rs`.
  - Run `cargo check` + targeted tests.
- [ ] Create `entrypoint/typescript.rs`; move `detect_typescript_entrypoints` (make `pub(super)`), `parse_typescript_handlers`, `parse_typescript_tests`.
  - Add `mod typescript; pub use typescript::detect_typescript_entrypoints;` to `entrypoint.rs`.
  - Run `cargo check` + targeted tests.
- [ ] Create `entrypoint/python.rs`; move `detect_python_entrypoints` (make `pub(super)`), `is_python_handler_decorator`, `parse_python_decorators`.
  - Add `mod python; pub use python::detect_python_entrypoints;` to `entrypoint.rs`.
  - Run `cargo check` + targeted tests.
- [ ] Verify `entrypoint.rs` now contains only shared types (`EntrypointKind`, `EntrypointStats`, `SymbolClassification`), `mod` declarations, and `pub use` re-exports.

Definition of done: All language detectors in their own modules; facade holds shared types and re-exports; clean compile.

## Phase 3: Test Relocation

- [ ] Move Rust-specific `#[cfg(test)]` fixtures and tests to `entrypoint/rust.rs`.
- [ ] Move TypeScript-specific fixtures and tests to `entrypoint/typescript.rs`.
- [ ] Move Python-specific fixtures and tests to `entrypoint/python.rs`.
- [ ] Keep any multi-language tests and `EntrypointKind` round-trip tests in the facade's `#[cfg(test)]` block.
- [ ] Run `cargo test index::entrypoint` and confirm the count matches the Phase 0 baseline.

Definition of done: Tests co-located with production code; count at baseline; all pass.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit: `changeguard ledger commit <tx-id> --summary "Completed Track GF13: entrypoint detectors split by language" --reason "1,045-line file with 3 independent language detectors split to mirror RE5/GF9/GF10 structure"`. If the git pre-commit hook removed the sidecar and `ledger status` still shows 1 pending after the git commit, run `ledger commit` again immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.
- [ ] Mark all tasks `- [x]` in this plan and set Status: Completed in `conductor/conductor.md`.

Definition of done: Full gates pass; installed binary matches source; ledger clean; conductor registry current.

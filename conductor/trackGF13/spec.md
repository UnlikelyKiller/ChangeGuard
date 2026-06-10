# Track GF13: Entrypoint Language Detector Split

## Objective

Split `src/index/entrypoint.rs` (1,045 lines, ~798 production) by language. The file contains three independent language-specific entrypoint detectors — Rust, TypeScript, and Python — each with its own attribute/decorator parsing logic and heuristics. The shared types (`EntrypointKind`, `EntrypointStats`, `SymbolClassification`) are small and serve all three detectors, making a natural facade.

This decomposition completes the language-module symmetry established by RE5 (Rust parser) and GF9/GF10 (Python/TypeScript parsers): the entrypoint detection layer now mirrors the extraction layer structure.

## Evidence

- 1,045 lines total; ~798 production (remainder is `#[cfg(test)]` test fixtures)
- Three independent language detectors identified via function inventory:
  - Rust: `detect_rust_entrypoints` (~167 lines), `is_rust_handler_attr`, `parse_rust_attributes`, `extract_rust_attr_path` (~115 lines combined)
  - TypeScript: `detect_typescript_entrypoints` (~103 lines), `parse_typescript_handlers`, `parse_typescript_tests` (~181 lines combined)
  - Python: `detect_python_entrypoints` (~127 lines), `is_python_handler_decorator`, `parse_python_decorators` (~106 lines combined)
- Shared types: `EntrypointKind`, `EntrypointStats`, `SymbolClassification` (~66 lines)
- No cross-language dependencies between the three detector groups

## Scope

Facade pattern: keep `src/index/entrypoint.rs` as the facade file and add a sibling `src/index/entrypoint/` directory (GF8 `dead_code.rs` pattern). `mod rust;` declared inside `entrypoint.rs` resolves to `entrypoint/rust.rs`. No rename to `entrypoint/mod.rs` at any point — having both files is E0761.

| Module | Assigned items |
|---|---|
| `entrypoint.rs` (facade) | `EntrypointKind`, `EntrypointStats`, `SymbolClassification` + `mod` declarations + `pub use` re-exports |
| `entrypoint/rust.rs` | `detect_rust_entrypoints`, `is_rust_handler_attr`, `parse_rust_attributes`, `extract_rust_attr_path` |
| `entrypoint/typescript.rs` | `detect_typescript_entrypoints`, `parse_typescript_handlers`, `parse_typescript_tests` |
| `entrypoint/python.rs` | `detect_python_entrypoints`, `is_python_handler_decorator`, `parse_python_decorators` |

All three public detection functions (`detect_rust_entrypoints`, `detect_typescript_entrypoints`, `detect_python_entrypoints`) remain reachable at their existing import paths via facade re-exports. Shared types stay in the facade — child modules reference them as `super::EntrypointKind` etc. (children can access parent items, including private ones).

Private helpers within each language module use `pub(super)` if needed for test access, otherwise stay private.

## Non-Goals

- No changes to detection logic, attribute sets, or heuristic rules.
- No changes to `EntrypointKind` variants or their string serialization.
- No call site migration.
- No touching `.changeguard` state files.

## Implementation Notes

- The `#[cfg(test)]` section (begins line 799, ~246 lines) contains per-language fixtures embedded as raw strings (the fixture code sits at column 0 inside `r#"..."#` literals — do not mistake it for top-level code). Move each fixture to its corresponding language module's `#[cfg(test)]` block.
- `SymbolClassification` is a struct with `pub` fields used only within the `index` module — confirm no external callers before narrowing visibility.
- No naming collision: `src/index/entrypoint/python.rs` and `src/index/languages/python/` are separate module trees (`index::entrypoint::python` vs `index::languages::python`).
- Moved detector functions become `pub(super)` in their child modules; the facade re-exports them `pub`.

## Verification Strategy

Targeted (run after each module move):
- `cargo check --all-targets --all-features`
- `cargo test index::entrypoint`

Final:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/index/entrypoint.rs` contains shared types, `mod` declarations, and re-exports only.
- Each language detector lives in its own module under `src/index/entrypoint/`.
- `detect_rust_entrypoints`, `detect_typescript_entrypoints`, `detect_python_entrypoints` remain importable from `crate::index::entrypoint`.
- `EntrypointKind`, `EntrypointStats`, `SymbolClassification` remain importable from `crate::index::entrypoint`.
- All existing tests pass.
- Full verification and reinstall pass.
- Ledger transaction committed; `changeguard ledger status --compact` shows `0 pending, 0 unaudited drift`.

## Risks

- Test fixture move: the large test section has per-language fixtures embedded together. Moving them alongside their language module is straightforward but requires care to not split a fixture used by multiple tests.
- `parse_rust_attributes` returns a `HashMap<String, Vec<String>>` — confirm this is not a public type alias before narrowing the visibility of the function.

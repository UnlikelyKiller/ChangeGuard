# Track GF9: Python AST Parser Extraction

## Objective

Split `src/index/languages/python.rs` (1,471 lines, ~1,007 production) into focused extraction modules by concern, directly mirroring the RE5 Rust language decomposition. The file houses six independent extraction pipelines in a single flat file — each pipeline is self-contained, testable in isolation, and easily extended independently once separated.

## Evidence

- 1,471 lines total; `#[cfg(test)]` begins at line 1008, so ~1,007 production lines
- Function inventory reveals six separable extraction concerns:
  - Symbol extraction — `extract_symbols` (~77 lines)
  - Route extraction — `extract_routes` + FastAPI/Flask helpers (~237 lines)
  - Call-edge extraction — `extract_calls` + graph-walk helpers (~125 lines)
  - Data model extraction — `extract_data_models` + Pydantic/attrs traversal (~159 lines)
  - Observability extraction — `extract_logging_patterns`, `extract_error_handling`, `extract_telemetry_patterns` + helpers (~360 lines)
  - Shared helper — `extract_py_attribute_name` (~15 lines; the only helper used across module boundaries)
- Helper usage audit (verified 2026-06-10 via grep):
  - `extract_py_attribute_name` (line 429): used by call-edge extraction (line 380) AND observability collectors (lines 673, 963) → genuinely shared → `common.rs`
  - `extract_py_attribute_object` (line 765): used only by observability collectors (lines 672, 962) → `observability.rs`
  - `is_in_py_test` (line 785): used only by observability collectors (lines 680–988) → `observability.rs`
  - `find_py_enclosing_function` (line 444): used only by call-edge extraction (line 342) → `calls.rs`
- No code outside `src/index/languages/` imports `languages::python::*` directly (verified via grep) — the only consumer is the dispatcher in `src/index/languages/mod.rs`, so import-path risk is minimal.
- RE5 established the decomposition shape for `src/index/languages/rust/`: `calls.rs`, `symbols.rs`, `models.rs`, `routes.rs`, `observability.rs`, `common.rs`. GF9 applies the same module names to Python.
- GF10 (TypeScript) follows the same pattern; GF9 establishes the precedent first.

## Scope

Facade pattern: keep `src/index/languages/python.rs` as the facade file and add a sibling `src/index/languages/python/` directory. `mod symbols;` declared inside `python.rs` resolves to `python/symbols.rs` — this is exactly the GF8 pattern (`dead_code.rs` + `dead_code/` directory). No file rename is needed at any point, which avoids the E0761 ambiguous-module error that a `python/mod.rs` conversion would risk mid-track.

| Module | Assigned functions / constants |
|---|---|
| `python.rs` (facade) | `mod` declarations + `pub use` re-exports only (≤ 30 lines) |
| `python/symbols.rs` | `extract_symbols` |
| `python/routes.rs` | `extract_routes`, `detect_fastapi_routers`, `detect_flask_objects`, `collect_py_routes`, `extract_py_decorator_path`, `find_py_decorated_function_name`, `extract_flask_method_from_decorator`, `PY_HTTP_METHODS` |
| `python/calls.rs` | `extract_calls`, `collect_py_call_edges`, `find_py_enclosing_function` |
| `python/models.rs` | `extract_data_models`, `collect_py_data_models` |
| `python/observability.rs` | `extract_logging_patterns`, `collect_py_logging_patterns`, `extract_error_handling`, `collect_py_error_handling`, `extract_telemetry_patterns`, `collect_py_telemetry_patterns`, `extract_py_attribute_object`, `is_in_py_test` |
| `python/common.rs` | `extract_py_attribute_name` (only genuinely cross-module helper) |

All currently-public functions (`extract_symbols`, `extract_routes`, `extract_calls`, `extract_data_models`, `extract_logging_patterns`, `extract_error_handling`, `extract_telemetry_patterns`) remain reachable at their existing import path via `python.rs` re-exports.

Moved public functions become `pub(super)` or `pub(crate)` in their child modules as needed; the facade re-exports them `pub`.

## Non-Goals

- No logic changes — query strings, output types, and extraction behavior are frozen.
- No new extraction features.
- No migration of external call sites — facade re-exports keep paths stable.
- No renaming `python.rs` to `python/mod.rs` — the facade-file pattern is the repo convention (GF1 `packet.rs`, GF4 `db.rs`, GF8 `dead_code.rs`).
- No touching `.changeguard` state files.

## Implementation Notes

- Child modules can access private items of their parent module, so nothing in `python.rs` needs widened visibility for children to use it. Functions moved INTO children that the facade or siblings call need `pub(super)`.
- Run `cargo check --all-targets --all-features` after each module is moved before proceeding.
- `extract_py_attribute_name` must land in `common.rs` before `calls.rs` or `observability.rs` is split, since both consume it.
- Tree-sitter imports (`Parser`, `Query`, `QueryCursor`, `StreamingIterator`) should be duplicated per module rather than re-exported from `common.rs` — clarity over DRY at module boundaries.
- The existing `#[cfg(test)]` block (line 1008+) moves with its subjects: each test relocates to the module whose functions it exercises.

## Verification Strategy

Targeted (run after each module move):
- `cargo check --all-targets --all-features`
- `cargo test index::languages::python`

Final:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/index/languages/python.rs` contains only `mod` declarations and `pub use` re-exports (≤ 30 lines).
- Each extraction concern lives in its own module file under `src/index/languages/python/`.
- All existing public import paths (`crate::index::languages::python::extract_*`) compile without change.
- `cargo test index::languages::python` passes.
- Final verification and reinstall pass.
- Ledger transaction committed; `changeguard ledger status --compact` shows `0 pending, 0 unaudited drift`.

## Risks

- Shared helper ordering: `extract_py_attribute_name` is consumed by two destination modules. Move it to `common.rs` first to avoid intermediate broken states.
- Test relocation: some tests may exercise multiple concerns; keep any cross-concern test in the facade's `#[cfg(test)]` block rather than forcing it into one module.

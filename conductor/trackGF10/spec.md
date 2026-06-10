# Track GF10: TypeScript AST Parser Extraction

## Objective

Split `src/index/languages/typescript.rs` (1,362 lines, ~945 production) into focused extraction modules by concern, following the same facade-file + directory shape as GF9 (Python) and the module naming of RE5 (Rust). The file contains seven public extraction pipelines in a single flat module.

## Evidence

- 1,362 lines total; `#[cfg(test)]` begins at line 946, so ~945 production lines
- Public extraction functions:
  - `extract_symbols` (~85 lines)
  - `extract_routes` + Fastify/Express helpers (~119 lines)
  - `extract_calls` + call-graph traversal helpers (~179 lines)
  - `extract_data_models` + Zod/TypeORM traversal (~124 lines)
  - `extract_logging_patterns`, `extract_error_handling`, `extract_telemetry_patterns` + helpers (~370 lines)
- Helper usage audit (verified 2026-06-10 via grep — note this differs from naive expectations):
  - `extract_ts_member_name` (line 367): used by routes (line 137), calls (line 298), AND observability (lines 595, 770) → genuinely shared → `common.rs`
  - `extract_ts_object_name` (line 215): used by routes (line 138), observability (lines 594, 784), and `is_in_ts_test` (line 702) → genuinely shared → `common.rs`
  - `is_in_ts_test` (line 684): used only by observability collectors (lines 602–918) → `observability.rs`
  - `is_in_ts_test_from_line` (line 850): used only by telemetry (line 836) → `observability.rs`
  - `truncate_str` (line 938): used only by telemetry (line 906) → `observability.rs`
  - `extract_ts_string_literal` (line 234): used only by routes (line 162) → `routes.rs`
  - `find_ts_enclosing_function` (line 382): used only by calls (lines 273, 338) → `calls.rs`
- No code outside `src/index/languages/` imports `languages::typescript::*` directly (verified via grep) — only the dispatcher in `src/index/languages/mod.rs` consumes it.
- Dependencies: GF9 should be completed first to validate the facade pattern; GF10 applies it identically.

## Scope

Facade pattern: keep `src/index/languages/typescript.rs` as the facade file and add a sibling `src/index/languages/typescript/` directory (GF8 `dead_code.rs` pattern). No rename to `typescript/mod.rs` at any point.

| Module | Assigned functions / constants |
|---|---|
| `typescript.rs` (facade) | `mod` declarations + `pub use` re-exports only (≤ 30 lines) |
| `typescript/symbols.rs` | `extract_symbols` |
| `typescript/routes.rs` | `extract_routes`, `detect_fastify`, `collect_ts_routes`, `extract_ts_string_literal` |
| `typescript/calls.rs` | `extract_calls`, `collect_ts_call_edges`, `find_ts_enclosing_function` |
| `typescript/models.rs` | `extract_data_models`, `collect_ts_data_models` |
| `typescript/observability.rs` | `extract_logging_patterns`, `collect_ts_logging_patterns`, `extract_error_handling`, `collect_ts_error_handling`, `extract_telemetry_patterns`, `collect_ts_telemetry_patterns`, `is_in_ts_test`, `is_in_ts_test_from_line`, `truncate_str` |
| `typescript/common.rs` | `extract_ts_member_name`, `extract_ts_object_name` (the two genuinely cross-module helpers) |

All public functions remain reachable at their existing import paths via facade re-exports.

## Non-Goals

- No logic changes, query string modifications, or output type changes.
- No new extraction features.
- No call site migration — facade re-exports preserve paths.
- No renaming `typescript.rs` to `typescript/mod.rs`.
- No touching `.changeguard` state files.

## Implementation Notes

- Follow the same mechanical steps as GF9: create the directory, extract `common.rs` first, then move extraction modules one at a time, each followed by `cargo check`.
- Both `common.rs` helpers are consumed by three destination modules each — they MUST be extracted before any consumer module is split.
- `is_in_ts_test` calls `extract_ts_object_name` internally; after the split this becomes `use super::common::extract_ts_object_name;` inside `observability.rs`.
- The `tree_sitter_typescript::LANGUAGE_TYPESCRIPT` constant is local to each module that creates a `Parser` — duplicate imports, do not centralize.
- Child modules can access the parent's private items, so the facade needs no visibility changes for anything that temporarily remains in it during the migration.

## Verification Strategy

Targeted (run after each module move):
- `cargo check --all-targets --all-features`
- `cargo test index::languages::typescript`

Final:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/index/languages/typescript.rs` contains only `mod` declarations and `pub use` re-exports (≤ 30 lines).
- Each extraction concern lives in its own module file under `src/index/languages/typescript/`.
- All existing public import paths compile without change.
- `cargo test index::languages::typescript` passes at or above the baseline test count.
- Final verification and reinstall pass.
- Ledger transaction committed; `changeguard ledger status --compact` shows `0 pending, 0 unaudited drift`.

## Risks

- The shared-helper set here is larger than Python's and was misjudged once already during planning (the observability-only test helpers look shared but are not; the member/object-name extractors look local but are not). Re-verify the usage audit with grep before moving anything.
- TypeScript AST queries use `LANGUAGE_TYPESCRIPT`; confirm no module needs `LANGUAGE_TSX` before assuming a single language constant per module.

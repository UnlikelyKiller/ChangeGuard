# Track J7: Dead-Code False Positive Filtering

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard dead-code` produces high false-positive rates because `is_entrypoint()` and the symbol scan do not filter:

1. **Test functions** — `#[test]` functions are by definition only called by the test harness, not by production code. They will always appear "unreachable" in graph analysis.
2. **Module re-exports** — `pub use inner::Foo;` items are graph leaves (they forward, not define) and appear unreachable even though they are the public API surface.
3. **Feature-gated items** — `#[cfg(feature = "...")]` items may be unreachable in the default feature set but reachable with the feature enabled. Flagging them as dead code without noting the gate is misleading.
4. **Derive macros and trait impls** — `impl Serialize for Foo` is called by serde-generated code, not by direct function calls. The KG may not model macro invocation edges.

The result: dead-code output is not actionable because engineers must manually filter noise.

## Fix Strategy
Add a multi-stage filter pipeline in `src/impact/analysis/dead_code.rs` `scan_repo()`:

1. **Test filter**: Exclude any symbol whose `symbol_kind` is `TEST` or whose name begins with `test_` or contains `#[test]` annotation in the raw source line.
2. **Re-export filter**: Exclude any symbol whose `symbol_kind` is `RE_EXPORT` or `USE_ITEM`.
3. **Feature gate annotation**: For symbols with a `cfg_feature` attribute, do not exclude them but annotate the result: `DeadSymbol::maybe_dead_feature_gated(feature: String)`.
4. **Entrypoint expansion**: Extend `is_entrypoint()` to also return `true` for:
   - Symbols with `symbol_kind` = `PROC_MACRO` or `DERIVE_MACRO`
   - Symbols where `name` matches `impl Trait for Type` pattern
   - Symbols with `#[no_mangle]` or `extern "C"` in their attributes

The KG's `symbol_kind` values must be inspected first (via `changeguard ask "what symbol_kind values exist in the KG?"`) to ensure the filter uses the correct enum strings.

## Scope of Changes

### 1. `src/impact/analysis/dead_code.rs`
- `is_entrypoint()`: add checks for proc macros, derive macros, extern functions, no_mangle
- `scan_repo()`: add filter pipeline after fetching all symbols; apply test/re-export/feature filters
- Add `DeadSymbol` annotation field `feature_gate: Option<String>` for feature-gated symbols

### 2. `src/commands/dead_code.rs` (or `src/commands/mod.rs`)
- In the output formatter: for feature-gated symbols, show `[cfg(feature = "...")]` annotation

### 3. Config (optional)
- Add `dead_code.exclude_tests: bool = true` to allow disabling the test filter if the user wants to audit test coverage.
- Add `dead_code.exclude_reexports: bool = true`.

## Success Criteria
- `changeguard dead-code` does not flag `#[test]` functions.
- `changeguard dead-code` does not flag `pub use` re-exports.
- Feature-gated symbols are shown with a `[cfg(feature = "...")]` annotation rather than a plain `dead` label.
- `extern "C"` and `#[no_mangle]` functions are not flagged as dead.
- Proc macros and derive macros are not flagged as dead.
- Total false positive count (verified manually against a known-good codebase) is < 10%.
- All existing dead-code tests pass.

## Files Changed
- `src/impact/analysis/dead_code.rs`
- `src/commands/dead_code.rs` (or output formatter)
- `src/config/model.rs` (optional config fields)
- `.changeguard/config.toml` (optional config fields)

## Edge Cases
- **`symbol_kind` values differ from assumptions**: Before implementing filters, query the KG (`changeguard ask`) for the actual `symbol_kind` enum values present. Use the actual values, not guesses.
- **Test module functions not annotated with `#[test]`**: Functions inside `#[cfg(test)]` modules are also test-only. Filter by module path containing `::tests::` or module kind = `TEST_MODULE`.
- **Bench functions**: `#[bench]` functions are similar to `#[test]`. Include in the test filter.
- **`dead_code.exclude_tests = false`**: When user opts in to seeing test dead code, do not apply the test filter. Still apply the re-export filter.
- **Symbols with both `#[test]` and `pub`**: The `pub` does not make a test function an entrypoint for production. Test filter takes precedence.
- **KG not indexed**: If the KG has no symbol data, `scan_repo()` returns an empty vec. Do not panic; return empty with a `warn!`.

## Definition of Done
- [ ] `changeguard dead-code` produces zero false positives for `#[test]` functions in this repo.
- [ ] `changeguard dead-code` produces zero false positives for `pub use` re-exports.
- [ ] Feature-gated symbols show `[cfg(feature = "...")]` annotation.
- [ ] `extern "C"` and `#[no_mangle]` functions are not flagged.
- [ ] Config fields `dead_code.exclude_tests` and `dead_code.exclude_reexports` work.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.

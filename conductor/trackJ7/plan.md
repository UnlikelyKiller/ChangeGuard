# Track J7 Plan: Dead-Code False Positive Filtering

## Steps

### Discovery (pre-implementation)
1. [ ] Run `changeguard ask "what symbol_kind values exist in the knowledge graph schema?"` to enumerate actual KG symbol_kind strings
2. [ ] Run `changeguard dead-code` on the current repo; capture output; manually label first 20 results as true/false positive
3. [ ] Document symbol_kind values used in the filter in a comment at the top of the relevant section

### Red Phase (failing tests)
4. [ ] Add unit test: build a mock symbol list including a `#[test]` function; assert it is excluded from dead-code output
5. [ ] Add unit test: build a mock symbol list including a `pub use` re-export; assert it is excluded
6. [ ] Add unit test: `extern "C"` function is recognized as entrypoint → not flagged
7. [ ] Add unit test: symbol with `cfg_feature = "some-feature"` → included in output with feature annotation
8. [ ] Add config test: `exclude_tests = false` → `#[test]` functions appear in output
9. [ ] Run CI gate — new tests expected to fail

### Green Phase (implementation)
10. [ ] Add `exclude_tests: bool` and `exclude_reexports: bool` to `DeadCodeConfig` with `serde(default)` = `true`
11. [ ] Add config fields to `.changeguard/config.toml` under `[dead_code]`
12. [ ] Extend `is_entrypoint()`: add `PROC_MACRO`, `DERIVE_MACRO`, extern function, `#[no_mangle]` checks using actual KG symbol_kind values discovered in step 1
13. [ ] In `scan_repo()`: after fetching symbols, apply filter pipeline:
    - If `exclude_tests`: filter out symbols where `symbol_kind` ∈ {TEST, BENCH} or name pattern `test_*` or module path contains `::tests::`
    - If `exclude_reexports`: filter out symbols where `symbol_kind` ∈ {RE_EXPORT, USE_ITEM}
14. [ ] Add `feature_gate: Option<String>` field to `DeadSymbol`; populate from symbol attributes
15. [ ] Update output formatter: show `[cfg(feature = "...")]` suffix for feature-gated symbols
16. [ ] Run `cargo build` — fix any type/import errors
17. [ ] Run CI gate — all tests expected to pass

### Verification
18. [ ] `cargo install --path .` to rebuild binary
19. [ ] `changeguard dead-code` → zero `#[test]` functions in output
20. [ ] `changeguard dead-code` → zero `pub use` re-exports in output
21. [ ] Feature-gated symbols show annotation
22. [ ] `changeguard verify` passes

### Finalization
23. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
24. [ ] `changeguard ledger commit` with summary and reason

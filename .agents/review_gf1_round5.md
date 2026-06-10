You are a senior Rust reviewer performing a read-only audit of a schema-stability golden test suite and a public-facade compatibility test for a large refactor.

## Context
The file `src/impact/packet.rs` was a 2064-line god-file containing all domain types for an impact packet. It has been decomposed into 8 submodules under `src/impact/packet/*.rs`, and `src/impact/packet.rs` is now a ~407-line facade that re-exports all public types.

The goal is to verify two things:
1. The golden tests are rigorous enough to catch any schema drift (field reordering, key renaming, missing/extra keys, default value changes) during future refactors.
2. The facade re-exports are comprehensive — all 39 public types remain importable through the old path.

## Files to review
Please review the following files (read-only) and report any findings. Focus on the test modules `schema_golden_tests` and `facade_compat_tests` inside `src/impact/packet.rs`.

- `src/impact/packet.rs` (facade + tests)
- `src/impact/packet/metadata.rs`
- `src/impact/packet/changed_file.rs`
- `src/impact/packet/risk.rs`
- `src/impact/packet/verification.rs`
- `src/impact/packet/coverage.rs`
- `src/impact/packet/surfaces.rs`
- `src/impact/packet/intelligence.rs`
- `src/impact/packet/serialization.rs`

## What to look for
1. **Golden test strength**: Does the test assert exact key sets for the top-level object and representative nested objects recursively? Does it verify emitted field order without relying on `serde_json::Value` map ordering (which may not preserve insertion order)? Does it assert `skip_serializing_if` omission behavior and `#[serde(default)]` fallback semantics?
2. **Facade completeness**: Does the `test_public_facade_imports_work` test instantiate every public type from the submodules? Are any types missing?
3. **Serde contract preservation**: Are `rename_all`, `skip_serializing_if`, `default`, and custom deserializers preserved after the move?
4. **Any regressions**: Are there any broken imports, lost derives, or changed visibilities?

## Expected outcome
Return either:
- **CLEAR** — no actionable findings.
- **ACTIONABLE: <list>** — specific findings with line references.

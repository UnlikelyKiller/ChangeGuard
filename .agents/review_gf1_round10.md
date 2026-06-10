You are a senior Rust reviewer performing a read-only audit of a schema-stability golden test suite and a public-facade compatibility test for a large refactor.

## Context
The file `src/impact/packet.rs` was a 2064-line god-file containing all domain types for an impact packet. It has been decomposed into 8 submodules under `src/impact/packet/*.rs`, and `src/impact/packet.rs` is now a ~407-line facade that re-exports all public types.

## Previous review findings (Round 9)
One actionable item remained:
- The facade-compat check was only a positive smoke test. It did not lock the negative contract that internal submodule paths and helper symbols must stay unimportable. A compile-fail test was needed to assert that paths like `crate::impact::packet::changed_file::ChangedFile` and `deserialize_score` fail to compile.

## Changes since Round 9
- Added `trybuild` dev-dependency.
- Created `tests/compile_fail/private_submodule_path.rs` which attempts `use changeguard::impact::packet::changed_file::ChangedFile;` and fails with `error[E0603]: module `changed_file` is private`.
- Created `tests/compile_fail/private_helper_leak.rs` which attempts `use changeguard::impact::packet::serialization::deserialize_score;` and fails with `error[E0603]: module `serialization` is private`.
- Added `tests/compile_fail.rs` runner using `trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs")`.
- Generated and committed `.stderr` files for deterministic trybuild matching.

## Files to review
Please review the following files (read-only) and report any remaining findings.

- `src/impact/packet.rs` (facade + tests)
- `src/impact/packet/metadata.rs`
- `src/impact/packet/changed_file.rs`
- `src/impact/packet/risk.rs`
- `src/impact/packet/verification.rs`
- `src/impact/packet/coverage.rs`
- `src/impact/packet/surfaces.rs`
- `src/impact/packet/intelligence.rs`
- `src/impact/packet/serialization.rs`
- `tests/compile_fail.rs`
- `tests/compile_fail/private_submodule_path.rs`
- `tests/compile_fail/private_helper_leak.rs`

## Expected outcome
Return either:
- **CLEAR** — no actionable findings.
- **ACTIONABLE: <list>** — specific findings with line references.

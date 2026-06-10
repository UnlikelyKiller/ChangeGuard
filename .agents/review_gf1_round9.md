You are a senior Rust reviewer performing a read-only audit of a schema-stability golden test suite and a public-facade compatibility test for a large refactor.

## Context
The file `src/impact/packet.rs` was a 2064-line god-file containing all domain types for an impact packet. It has been decomposed into 8 submodules under `src/impact/packet/*.rs`, and `src/impact/packet.rs` is now a ~407-line facade that re-exports all public types.

## Previous review findings (Round 8)
Two actionable items remained:
1. `pub mod` for each submodule exposed the internal file layout as public API (`crate::impact::packet::changed_file::ChangedFile`, etc.), undercutting the facade's purpose.
2. `pub use self::serialization::*;` leaked the implementation helper `deserialize_score` through the facade.

## Changes since Round 8
- Changed all `pub mod` declarations to `mod` (private submodules).
- Removed `pub use self::serialization::*;` entirely, so `deserialize_score` is no longer re-exported through the facade.
- Verified no other code in the crate imports from the submodule paths directly.

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

## Expected outcome
Return either:
- **CLEAR** — no actionable findings.
- **ACTIONABLE: <list>** — specific findings with line references.

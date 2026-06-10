You are a senior Rust reviewer performing a read-only audit of a schema-stability golden test suite and a public-facade compatibility test for a large refactor.

## Context
The file `src/impact/packet.rs` was a 2064-line god-file containing all domain types for an impact packet. It has been decomposed into 8 submodules under `src/impact/packet/*.rs`, and `src/impact/packet.rs` is now a ~407-line facade that re-exports all public types.

## Previous review findings (Round 6)
One actionable item remained:
- `CIGate` lacked deserialize-side default-fallback coverage for the omitted `Vec` fields (`artifacts` and `release_gates`).

## Changes since Round 6
Added inside `test_nested_ci_gate_field_order_and_omission`:
```rust
let minimal = r#"{"platform":"github","jobName":"test","trigger":null}"#;
let parsed: CIGate = serde_json::from_str(minimal).unwrap();
assert!(parsed.artifacts.is_empty());
assert!(parsed.release_gates.is_empty());
```

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

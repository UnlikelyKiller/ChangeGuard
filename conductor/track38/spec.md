# Track 38: Complexity & Temporal Hardening

## Objective
Close the remaining Track 31 audit gaps from `docs/audit4.md`: document the `arborist-metrics` decision, strengthen complexity degradation coverage, add real git-backed temporal traversal tests, and add deterministic hotspot tests.

## Requirements
- Document the `arborist-metrics` spike decision in `docs/architecture/`.
- Represent unsupported complexity inputs explicitly as not applicable.
- Add TypeScript, syntax-error, unsupported-language, and large-file cap tests.
- Add a real git fixture test for first-parent temporal history extraction.
- Add hotspot tests for normalized multiplication, deterministic path tie-breaking, filters, JSON serializability, and SQLite row error propagation.
- Keep `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and all tests green.

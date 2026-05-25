# Track CR8 Plan: Escape Symbol Names in Cozo Queries

## Phase 1: Implementation
- [x] Added `pub fn escape_cozo_string(s: &str) -> String` helper in `src/commands/ask.rs`.
- [x] Escapes single quotes by doubling them (`'` → `''`).
- [x] Escapes backslashes by doubling them (`\` → `\\`).
- [x] All symbol interpolations in Datalog queries now use `escape_cozo_string`.

## Phase 2: Testing & Verification
- [x] Unit tests added in `tests/cli_verify.rs` (`escape_cozo_string_tests` module).
- [x] `cargo test` passes.

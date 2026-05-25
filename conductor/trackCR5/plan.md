# Track CR5 Plan: CLI & Process Hardening Test Coverage

## Phase 1: Implementation
- [x] Add integration tests for `verify --dry-run` in `tests/cli_verify.rs`.
- [x] Add integration tests for `verify --health` (pass and fail cases) in `tests/cli_verify.rs`.
- [x] Add CR4 regression test for env-var prefix command parsing in `tests/cli_verify.rs`.
- [x] Add CR8 unit tests for `escape_cozo_string` in `tests/cli_verify.rs`.

## Phase 2: Testing & Verification
- [x] `cargo test` passes with all new tests green.

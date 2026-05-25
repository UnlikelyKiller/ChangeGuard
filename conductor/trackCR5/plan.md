# Track CR5 Plan: CLI & Process Hardening Test Coverage

## Phase 1: Verify & Config Integration Testing
- [ ] Add tests in `tests/cli_verify.rs` for:
  - [ ] `verify --health` returning correct executable statuses.
  - [ ] `verify --dry-run` successfully displaying plan without launching execution.
- [ ] Add tests in `tests/cozo_schema_migration.rs` or `tests/cli_init.rs` for `config view` with `--section` and `--key` filters.

## Phase 2: Refine Force-Unlock Process Matching
- [ ] In `src/commands/update.rs`, refine the process scanning logic to check the current executable path or command-line parameters (if accessible) to ensure the target process is operating on the same workspace before invoking `.kill()`.
- [ ] Add mock processes verification tests on Windows.

## Phase 3: Final Verification
- [ ] Run `cargo test` to execute the full integration test suite.

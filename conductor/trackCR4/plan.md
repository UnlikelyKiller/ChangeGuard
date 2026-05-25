# Track CR4 Plan: Align Health Check Command Parsing

## Phase 1: Implementation
- [x] Add `extract_executable()` helper in `src/commands/verify.rs` that skips leading `KEY=value` tokens and strips surrounding quotes.
- [x] Replace the `split_whitespace().next()` call in the `--health` path with `extract_executable(&step.command)`.

## Phase 2: Testing & Verification
- [x] Regression test added: `test_verify_health_check_env_prefix_command` in `tests/cli_verify.rs`.
- [x] `cargo test` passes.

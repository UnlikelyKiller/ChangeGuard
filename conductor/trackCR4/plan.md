# Track CR4 Plan: Align Health Check Command Parsing

## Phase 1: Implementation
- [ ] Inspect executable parsing logic in `src/verify/runner.rs` (e.g., shell classification, quoted extraction).
- [ ] Share or duplicate the robust command parsing logic in `src/commands/verify.rs` for the `--health` loop.
- [ ] Correctly strip leading quotes or environment variable overrides (e.g., `ENV_VAR=value executable`) before verifying if the binary is present on `PATH`.

## Phase 2: Testing & Verification
- [ ] Run `changeguard verify --health` with:
  - [ ] Quoted commands (e.g., `"C:\Program Files\Git\cmd\git.exe" status`).
  - [ ] Env-prefixed commands (e.g., `RUST_BACKTRACE=1 cargo test`).
  - [ ] Built-in shell commands.
- [ ] Confirm no false missing binary warnings are outputted.

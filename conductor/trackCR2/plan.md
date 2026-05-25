# Track CR2 Plan: Enforce Signature Verification Failures

## Phase 1: Implementation
- [x] Modify `verify_ledger_signatures` in `src/commands/verify.rs` to treat unsigned committed entries as failures.
- [x] Set `all_valid = false` for entries with no signature.
- [x] Update the final error message to reflect both invalid and unsigned entries.

## Phase 2: Testing & Verification
- [x] Tests added in `tests/cli_verify.rs` covering `--signatures` behavior.
- [x] `cargo test` passes.

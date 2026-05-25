# Track CR2 Plan: Enforce Signature Verification Failures

## Phase 1: Implementation
- [ ] Modify `verify_ledger_signatures` in `src/commands/verify.rs` to track if any unsigned committed entry is found.
- [ ] If unsigned entries are found, set `all_valid = false` so that the verification fails overall.
- [ ] Ensure the error message returned is clear and indicates that unsigned entries are not permitted under the signature verification policy.

## Phase 2: Testing & Verification
- [ ] Add integration tests in a new or existing test file (`tests/ledger_crypto.rs` or `tests/cli_verify.rs`) to verify:
  - [ ] A ledger containing only signed entries succeeds (`exit 0`).
  - [ ] A ledger containing one or more unsigned committed entries fails (`exit 1`).
  - [ ] A ledger containing malformed signatures fails (`exit 1`).
- [ ] Verify using `cargo test`.

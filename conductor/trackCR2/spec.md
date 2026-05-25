# Track CR2: Enforce Signature Verification Failures

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
Currently, `changeguard verify --signatures` checks the signatures of committed transactions. If an entry has no signature, it prints a warning (`UNSIGNED`) but does not cause the verification process to fail (exit code is still `0`). This prevents enforcing ledger signing requirements.

## Objective
Enforce that `verify --signatures` returns a non-zero exit code (failure) when unsigned committed ledger entries exist, specifically in configurations where signing is enabled or generally when auditing signature requirements.

## Scope
- Modify signature validation in `src/commands/verify.rs` to mark unsigned committed entries as a verification failure.
- Ensure that the command exits with an error status code if unsigned entries are found.
- Implement tests verifying behavior with unsigned and invalid signatures.

## Success Criteria
- [ ] Running `changeguard verify --signatures` on a ledger containing unsigned entries returns a non-zero exit code.
- [ ] Valid signed entries pass without error.
- [ ] Integration tests are added to verify exit code behavior.

## Definition of Done
- [ ] `src/commands/verify.rs` modified to treat unsigned committed entries as a failure condition.
- [ ] CLI exit code status aligned with verification results.
- [ ] Signature integration tests added in `tests/cli_verify.rs` or `tests/ledger_crypto.rs`.
- [ ] `cargo test` passes.

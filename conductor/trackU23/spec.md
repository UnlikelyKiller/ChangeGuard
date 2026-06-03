# Track U23: Signature Enforcement in Pre-Push Hook

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P0 — Security / Data Integrity

---

## Problem Statement

While `changeguard verify --signatures` validates the integrity and cryptographic signatures of the ledger, the standard pre-push git hook only runs `changeguard ledger status --compact --exit-code`, which does not check signature validity. This allows commits with corrupted/invalid signatures to bypass the gate.

## Acceptance Criteria

**AC1:** A new `--verify-signatures` (or `--strict`) flag is added to the `ledger status` command.
**AC2:** When `--verify-signatures` is passed, `ledger status` performs cryptographic verification of all ledger entries. If any invalid signatures are found, the command exits with a non-zero exit code.
**AC3:** The pre-push hook configuration template is updated to include signature checks in its execution gate.

## Design Notes

- Update CLI definitions in `src/cli.rs`.
- Integrate signature verification logic (from `verify --signatures` module) into `execute_ledger_status`.
- Ensure output remains clean and compact for CI pipelines.

## Verification

- Modify a signature manually in a test database to make it invalid.
- Run `changeguard ledger status --compact --verify-signatures --exit-code`.
- Verify the command exits with a non-zero code.

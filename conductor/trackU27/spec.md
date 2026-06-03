# Track U27: Ledger Subcommand Parity & GC Mode Validation

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P1 — CLI Parity / Robustness

---

## Problem Statement

1. Some subcommands like `ledger resume` and `ledger note` are referenced but not present.
2. `ledger gc --force` is silently ignored when no GC mode is specified.

## Acceptance Criteria

**AC1:** If `ledger gc --force` is called without mode arguments (e.g. `--orphans`), it errors explicitly instead of doing nothing or printing "Please specify a GC mode".
**AC2:** Resolve the status of `ledger resume` and `ledger note` commands (either clean up docs or add subcommands).

## Design Notes

- In `src/commands/ledger.rs` (or gc handler), validate that `--force` requires a mode argument like `--orphans`.
- Coordinate documentation and CLI parsing logic.

## Verification

- Run `changeguard ledger gc --force` and check for the explicit error message.

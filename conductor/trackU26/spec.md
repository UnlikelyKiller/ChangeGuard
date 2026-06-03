# Track U26: Verify & Audit Output Cleanup

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P1 — UX / Output Noise

---

## Problem Statement

1. `verify --signatures` prints a global trailing error message even if all entries were successfully validated.
2. `audit --entity` streams diagnostic output from local LLM fallbacks into the CLI reason fields, cluttering output on non-interactive environments.

## Acceptance Criteria

**AC1:** `verify --signatures` suppresses the trailing error message on successful runs.
**AC2:** `audit --entity` separates or suppresses verbose LLM fallback warnings/logs from the core ledger audit reason display.

## Design Notes

- Update success status tracking in `verify_signatures` and avoid logging errors if `all_valid` is true.
- Adjust log levels or format logic in `audit --entity` to prevent diagnostic prints from contaminating stderr/stdout values.

## Verification

- Run `changeguard verify --signatures` on a valid ledger and verify no global failure message is shown.
- Run `changeguard audit --entity` under non-interactive modes and verify diagnostic lines are clean.

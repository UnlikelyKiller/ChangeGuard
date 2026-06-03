# Track U28: Init Storage Bootstrap

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P2 — CLI Ergonomics

---

## Problem Statement

When running `changeguard init`, the ledger storage database is not bootstrapped immediately. A user running `ledger status` immediately after `init` receives a "Storage not initialized" error, requiring them to run a write command (like `scan`) first.

## Acceptance Criteria

**AC1:** `changeguard init` initializes the local SQLite database/ledger storage, writing a placeholder or base configuration record so that subsequent read operations (`ledger status`) work immediately.

## Design Notes

- In `src/commands/init.rs`, call the ledger database bootstrap logic to initialize tables and write a seed record.

## Verification

- Run `changeguard init` on a clean directory and immediately run `changeguard ledger status`. Verify it returns 0 pending transactions rather than an initialization error.

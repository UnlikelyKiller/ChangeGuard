# Track U24: Reset Safety & Path Hygiene

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P0 — Security / Data Integrity

---

## Problem Statement

1. Running `changeguard reset --all --yes` destroys `.changeguard/` with no recovery, no pre-deletion diff, and no audit log.
2. Case-mismatched working trees (`C:\dev\changeguard` and `C:\dev\ChangeGuard`) lead to divergent `.git/hooks` paths and command confusion.

## Acceptance Criteria

**AC1:** A new `--dry-run` flag is added to the `reset` command.
**AC2:** Before executing any deletion, `reset` prints a clear list of directories and files to be removed, even when `--yes` is specified.
**AC3:** Path canonicalization is applied during repository discovery and environment setups, preventing case-mismatched path divergence under Windows.

## Design Notes

- In `src/commands/reset.rs`, list files to be deleted and print them. If `--dry-run` is active, skip actual removal.
- Standardize on path normalization using absolute, canonicalized, and case-resolved paths.

## Verification

- Run `changeguard reset --all --yes --dry-run` and verify that the files to be deleted are printed but not removed.

# Track U31: Update --binary Pre-flight

**Status:** ⏳ **Pending**
**Started:** None
**Owner:** None
**Priority:** P1 — Usability / Operational safety

---

## Problem Statement

Running `changeguard update --binary` installs/replaces the global binary with the current source build without displaying what target path it is replacing or warning the user.

## Acceptance Criteria

**AC1:** `update --binary` outputs the path of the binary that will be replaced.
**AC2:** The command supports `--dry-run`, detailing the planned copy operation without mutating the filesystem.

## Design Notes

- In `src/commands/update.rs`, query the target binary install directory (e.g. cargo bin path or configured project path).
- Print the target and source paths. Skip installation if `--dry-run` is active.

## Verification

- Run `changeguard update --binary --dry-run` and verify that the target path is printed but not replaced.

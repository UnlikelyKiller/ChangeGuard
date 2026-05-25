# Track CR9: Scope Windows Shadow Copies Cleanup

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
At startup, `src/main.rs` performs a sweep on Windows to clean up temporary shadow executables. However, the current cleanup logic removes any adjacent file matching `*.old.*.exe` in the directory, regardless of whether it was created by the active ChangeGuard binary instance or some unrelated application/executable. This broad matching criteria can result in the deletion of important files belonging to other binaries.

## Objective
Refine the Windows shadow copies cleanup process in `src/main.rs` to target only the shadow files belonging specifically to the current ChangeGuard binary.

## Scope
- Modify the startup cleanup logic in `src/main.rs`.
- Extract the current running executable name and use it to construct a specific file name pattern (e.g. `[exec_name].old.*.exe`).
- Restrict file deletion to only match this specific pattern rather than deleting all adjacent `*.old.*.exe` files.

## Success Criteria
- [ ] Startup cleanup only removes shadow files belonging to the running ChangeGuard binary name.
- [ ] Other adjacent files matching `*.old.*.exe` for different executables (e.g. `another_app.old.123.exe`) are preserved.
- [ ] Robust file system permissions/errors handling.

## Definition of Done
- [ ] Safe pattern matching implemented in `src/main.rs`.
- [ ] Verified manually or with unit tests matching different binary prefixes.
- [ ] `cargo test` passes.

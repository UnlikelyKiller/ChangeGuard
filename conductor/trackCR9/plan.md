# Track CR9 Plan: Scope Windows Shadow Copies Cleanup

## Phase 1: Implementation
- [ ] In `src/main.rs`, retrieve the current executable path using `std::env::current_exe()`.
- [ ] Extract the file stem/name of the current executable (e.g. `changeguard`).
- [ ] Construct the matching pattern specific to the executable: `[exec_stem].old.*.exe`.
- [ ] Refine the directory traversal and pattern matching logic to check against this executable-specific pattern.

## Phase 2: Testing & Verification
- [ ] Create mock files in a test directory representing:
  - [ ] A matching shadow file (e.g. `changeguard.old.123.exe`).
  - [ ] A non-matching shadow file (e.g. `otherapp.old.123.exe`).
- [ ] Run the cleanup function on this test directory and verify that only the matching shadow file is removed, and the other is preserved.

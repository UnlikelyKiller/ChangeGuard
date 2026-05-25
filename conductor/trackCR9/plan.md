# Track CR9 Plan: Scope Windows Shadow Copies Cleanup

## Phase 1: Implementation
- [x] Modified `sweep_stale_old_binaries()` in `src/main.rs` to derive the cleanup prefix from `current_exe().file_stem()`.
- [x] Prefix is now `"{stem}.old."` (e.g. `changeguard.old.`) instead of the hardcoded `"changeguard.old."`.
- [x] Fallback to `"changeguard.old."` if `current_exe()` cannot be resolved.

## Phase 2: Testing & Verification
- [x] `cargo test` passes — existing tests remain green.

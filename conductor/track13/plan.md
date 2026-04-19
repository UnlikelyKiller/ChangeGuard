# Plan: Track 13 - Final Integration and Reset Command

### Phase 14: Final Integration
- [ ] Task 13.1: Implement `src/commands/reset.rs`.
  - [ ] Add `execute_reset` with `--force` support.
  - [ ] Use `std::fs::remove_dir_all`.
- [ ] Task 13.2: Register `reset` subcommand in `src/cli.rs`.
- [ ] Task 13.3: Refactor `src/cli.rs` help messages for all commands.
- [ ] Task 13.4: Implement full integration test in `tests/e2e_flow.rs`.
  - [ ] Use `DirGuard` and `tempdir` for sandboxing.
  - [ ] Sequence all core commands.
- [ ] Task 13.5: Final verification with `cargo test -j 1 -- --test-threads=1`.
- [ ] Task 13.6: Update project documentation (`Docs/Engineering.md`) if necessary.

# Plan: Track 12 - UI/UX Refinement

### Phase 13: UI/UX
- [ ] Task 12.1: Add `comfy-table` and `indicatif` to `Cargo.toml`.
- [ ] Task 12.2: Implement `src/ui/mod.rs` with common styling helpers.
- [ ] Task 12.3: Refactor `src/commands/scan.rs` to use `comfy-table` for change lists.
- [ ] Task 12.4: Refactor `src/commands/impact.rs` to use `comfy-table` for risk reasons and `indicatif` for symbol extraction progress.
- [ ] Task 12.5: Update `src/commands/ask.rs` to use `indicatif` spinner while waiting for Gemini.
- [ ] Task 12.6: Audit and enhance error messages in all modules.
- [ ] Task 12.7: Final verification with `cargo test -j 1 -- --test-threads=1`.

# Track X15 Plan: `watch` Startup Banner and Exit Handling

## Phase 1 — Implementation

### Startup banner
- [ ] 1. In `src/commands/watch.rs` `execute_watch`, immediately after resolving the repo root path and before starting the watcher loop:
  ```rust
  println!(
      "{} {}  {}",
      "Watching:".bold().green(),
      root_path.display(),
      "(press Ctrl+C to stop)".dimmed()
  );
  ```

### Ctrl+C exit code
- [ ] 2. Ensure the `ctrlc` handler sets an `AtomicBool` flag and calls `std::process::exit(0)` or prints before returning. Check existing handler in `execute_watch`.
- [ ] 3. Add `println!("Watch stopped.");` before exit in the Ctrl+C handler.

### `.changeguard/state/` ignore
- [ ] 4. In the debounce event filter, check if the changed path starts with `root_path.join(".changeguard")`. If so, skip the event:
  ```rust
  if path.starts_with(root_path.join(".changeguard")) {
      continue;
  }
  ```

## Phase 2 — Verification
- [ ] 5. Start `changeguard watch`, confirm immediate startup banner.
- [ ] 6. Press Ctrl+C, confirm exit code 0 (`$LASTEXITCODE` in PowerShell = 0) and "Watch stopped." message.
- [ ] 7. Touch `.changeguard/state/ledger.cozo`, confirm no re-analysis fires.
- [ ] 8. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 9. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 10. Run `cargo fmt --all -- --check` — clean.
- [ ] 11. Update `conductor/conductor.md` status to Completed.

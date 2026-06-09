# Track Y4 Plan: Progress Feedback for Blocking Operations

## Phase 1 — `ask` LLM Progress
- [ ] 1. In `src/commands/ask.rs`, before the local model completion call in `execute_ask`, add:
  ```rust
  if !json {
      eprintln!("Contacting LLM...");
  }
  ```
- [ ] 2. After the response is received and printed, the message is naturally replaced by the answer.

## Phase 2 — `verify` Command Progress
- [ ] 3. In `src/commands/verify.rs` `execute_verify`, before `run_verification_command`:
  ```rust
  if !json {
      eprintln!("Running: {} ...", command);
  }
  ```
- [ ] 4. The result line `SUCCESS / FAILURE` for `<name>` follows on stdout.

## Phase 3 — `index --semantic` Progress
- [ ] 5. In `src/commands/index.rs` semantic path, collect the file list and print count before processing:
  ```rust
  if !json {
      eprintln!("Embedding {} files with local model...", files.len());
  }
  ```
- [ ] 6. After completion, print a brief summary line to stderr.

## Phase 4 — Stale Index Non-Interactive Guard
- [ ] 7. Add `CHANGEGUARD_NON_INTERACTIVE` env-var check at the top of `src/index/staleness.rs` `warn_if_stale` / `prompt_reindex`:
  ```rust
  fn is_non_interactive() -> bool {
      std::env::var("CHANGEGUARD_NON_INTERACTIVE")
          .ok()
          .map_or(false, |v| !v.is_empty())
  }
  ```
- [ ] 8. When `is_non_interactive()`, skip the `inquire::Confirm` prompt and return a "stale index" warning directly.

## Phase 5 — Verification
- [ ] 9. Run `changeguard ask "test"` in human mode — confirm "Contacting LLM..." appears.
- [ ] 10. Run `CHANGEGUARD_NON_INTERACTIVE=true changeguard ask --semantic "test"` — confirm no interactive prompt.
- [ ] 11. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 12. Run `cargo nextest run --test integration` — all pass.
- [ ] 13. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 14. Run `cargo fmt --all -- --check` — clean.
- [ ] 15. Update `conductor/conductor.md` status to Completed.
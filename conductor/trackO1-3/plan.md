# Plan: Track O1-3 (Git Hook Integration & UX Logic)

- [ ] 1. Modify `src/commands/init.rs` to create `.git/hooks/commit-msg`.
- [ ] 2. Create the hook script payload to invoke `changeguard internal hook-commit-msg <msg_file>`.
- [ ] 3. Add the `internal hook-commit-msg` CLI command in `src/cli.rs`.
- [ ] 4. Implement the execution flow: diff capture -> `IntentDrafter` -> TUI launch or silent commit.
- [ ] 5. Implement the adaptive bypass logic by tracking recent skips in `.changeguard/state/skip_history.json`.
- [ ] 6. Add `intent.required` to `Config` (`src/config/model.rs`).
- [ ] 7. Add integration tests verifying the bypass logic and hook execution.
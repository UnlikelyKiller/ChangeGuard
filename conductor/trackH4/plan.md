# Plan: Track H4 (Windows Deployment Safety)

- [ ] 1. Identify the location of the current executable in `src/commands/update.rs`.
- [ ] 2. Implement a Windows-specific rename logic using `std::fs::rename`.
- [ ] 3. Wrap the `cargo install` call in the rename/replacement logic.
- [ ] 4. Add a post-install cleanup step to remove the renamed backup.
- [ ] 5. Manually verify by running `update --binary` on the local machine.

# Plan: Track H5 (Process & Path Hardening)

- [ ] 1. Audit the PID file writing logic in `src/commands/viz.rs`.
- [ ] 2. Ensure the PID file path is normalized using `camino` and exists before writing.
- [ ] 3. Implement an encoding check in `src/index/walker.rs` or the file reader utility.
- [ ] 4. Add a "Warning: Non-UTF8 file detected" diagnostic to the indexer output.
- [ ] 5. Verify the fix by indexing a file explicitly saved with UTF-16 encoding.

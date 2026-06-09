## Plan: Track 2: Doctor and Platform Detection
### Phase 3: Platform Detection and Diagnostics
- [ ] Task 3.1: Scaffold the `src/platform/` module (`mod.rs`, `detect.rs`, `shell.rs`, `paths.rs`, `env.rs`) and set up unit test skeletons.
- [ ] Task 3.2: Write tests for identifying Windows, Linux, and WSL environments, then implement `src/platform/detect.rs` using TDD. Verify with `cargo test -j 1`.
- [ ] Task 3.3: Write test cases for recognizing WSL mounted paths (e.g., `/mnt/c/`) and native paths. Implement `src/platform/paths.rs` classification logic. Verify with `cargo test -j 1`.
- [ ] Task 3.4: Write tests mocking environment variables to detect the active shell (PowerShell, bash, etc.). Implement `src/platform/shell.rs` to pass these tests.
- [ ] Task 3.5: Write fake executable path tests. Implement `src/platform/env.rs` logic to verify the existence of `git` and `gemini` / `gemini-cli` binaries in the system path. Verify with `cargo test -j 1`.
- [ ] Task 3.6: Write integration test `tests/cli_doctor.rs` to assert the overall structure, missing tools reporting, and deterministic output of the `doctor` subcommand.
- [ ] Task 3.7: Implement `src/commands/doctor.rs`. Aggregate data from `src/platform/` and format output. Provide `miette` based error reporting. Warn about cross-environment misconfigurations.
- [ ] Task 3.8: Register the `doctor` command within `src/cli.rs` and `src/main.rs`.
- [ ] Task 3.9: Ensure full test suite passes natively. Run `cargo test -j 1`.

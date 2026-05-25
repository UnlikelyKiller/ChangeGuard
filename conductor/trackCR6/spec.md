# Track CR6: Strong Process Validation for Viz Server Stop

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
In `src/commands/viz_server.rs` on Windows, the `--stop` command scans running processes using `tasklist` and checks if the output line contains the substring `"changeguard"`. It then terminates that process PID using `taskkill /F`. A simple substring check is vulnerable to false positives and PID reuse issues, as other running processes might have `"changeguard"` in their name or command-line parameters, leading to unintended process termination.

## Objective
Implement a more precise and safe process identification check in `viz-server --stop` on Windows, ensuring that we check the process image path or verify the target binary name precisely before invoking `taskkill`.

## Scope
- Modify the Windows process scanning and filtering logic in `src/commands/viz_server.rs`.
- Extract and match the exact image name and/or check the executable path of the target processes to ensure it matches the expected `changeguard.exe`.
- Prevent accidental termination of processes containing `"changeguard"` as a substring of their command-line arguments (e.g. loggers, text editors, or sub-processes) unless they are the actual viz server binary.

## Success Criteria
- [ ] Running `changeguard viz-server --stop` on Windows only kills the actual viz server process.
- [ ] No processes matching loosely on substring (e.g., a file named `not_changeguard.exe` or a shell running a command) are terminated.
- [ ] Robust error handling is implemented if process scanning fails.

## Definition of Done
- [ ] Safe image-name/path matching implemented in `src/commands/viz_server.rs` for Windows.
- [ ] Process scanning verified with concurrent test scenarios.
- [ ] `cargo test` passes.

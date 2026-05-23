# Track H5: Process & Path Hardening

## Objective
Harden PID management for background services on Windows and implement encoding detection to prevent indexing failures.

## Requirements
- **PID Discovery**: Fix the `viz-server --stop` logic. Ensure PIDs are correctly written to and read from the `.changeguard/tmp/` directory using Windows-compatible file paths.
- **Encoding Awareness**: Detect common Windows encodings (like UTF-16 from PowerShell redirection).
- **Graceful Failure**: If a file is not UTF-8, either attempt a lossy conversion or provide a clear diagnostic message to the user instead of a raw I/O error.

## Definition of Done (DoD)
- [ ] `changeguard viz-server --stop` successfully kills the background process on Windows.
- [ ] Files created via PowerShell (UTF-16) no longer crash the indexer; they are either indexed or skipped with a helpful warning.

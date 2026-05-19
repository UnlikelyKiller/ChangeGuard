# Specification: Named Pipe IPC Integration (Track B5)

## Overview
Provide low-latency, real-time sync with AI-Brains via Windows Named Pipes (`\\.\pipe\aibrains-sync`). This is the primary preferred transport over CLI polling.

## Architecture & SRP
- **Module**: `src/bridge/ipc.rs`
- **Constraints**: Tokio is feature-gated (`optional = true`) in ChangeGuard. To avoid forcing an async runtime into the core CLI, IPC must utilize standard library blocking I/O with rigid timeout protection.
- **Responsibility**: Establish connection, send payloads, and manage unresponsiveness.

## Requirements
- Open connection using `std::fs::OpenOptions::new().read(true).write(true).open(r"\\.\pipe\aibrains-sync")`.
- **Hang Prevention**: Since synchronous file opening can hang on Windows Named Pipes, wrap the open attempt inside a scoped thread. Return the handle over `std::sync::mpsc::channel` paired with `.recv_timeout(Duration::from_millis(200))`.
- **Fail-open**: If the pipe is unavailable or times out, immediately downgrade to the CLI fallback defined in Track B4 or proceed empty-handed without crashing.

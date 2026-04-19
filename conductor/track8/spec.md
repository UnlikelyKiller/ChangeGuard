# Technical Specification: Track 8 - Determinism Contract and Subprocess Control

## 1. Overview
This specification details the implementation of the "Execution Boundary" for bounded subprocess execution within Changeguard. The goal is to safely run verification commands or git subprocesses without risking application hangs. It includes strict timeout controls, deterministic output sorting, safe output capture, and mapping the results to the `ImpactPacket`.

## 2. Architecture & Design

### 2.1 Execution Boundary (`src/util/process.rs` and `src/verify/runner.rs`)
- **Process Spawning**: Subprocesses will be spawned using `std::process::Command`.
- **Timeout Control**: Execution will be wrapped in a timeout mechanism. A monitoring thread or channel-based wait will ensure that if the process exceeds its configured duration, it is forcibly killed.
- **Resource Limiting**: Limits will be placed on output buffers to prevent out-of-memory crashes when a command produces massive output (e.g., runaway logs). 

### 2.2 Subprocess Capture
- **Independent Streams**: `stdout` and `stderr` will be captured separately via pipes.
- **Output Truncation**: A constant byte limit (e.g., `MAX_OUTPUT_BYTES = 1024 * 1024` for 1MB) will be enforced. Any output exceeding this size will be truncated with a warning annotation appended to the string.
- **Encoding Safety**: Captured output will be processed with `String::from_utf8_lossy` to gracefully handle non-UTF-8 characters across different platforms.

### 2.3 Deterministic Output Sorting
- To maintain predictable state and consistent hashing for the `ImpactPacket`, any collections, lists of changed files, or tool execution results MUST be deterministically sorted (lexicographically or by predefined keys) prior to serialization.
- All lists inside `ImpactPacket` should apply a `.sort()` or `.sort_unstable()` before finalizing the struct.

### 2.4 Error Handling
- Use `miette` and `thiserror` for comprehensive error handling.
- Expected failure modes (e.g., Command Not Found, Timeout, Non-Zero Exit Code, IO Error) must be captured as explicit enum variants to provide actionable CLI diagnostics.

## 3. Data Structures

```rust
use std::time::Duration;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ProcessError {
    #[error("Command not found: {cmd}")]
    #[diagnostic(help("Ensure the executable is in your PATH and accessible."))]
    NotFound { cmd: String },

    #[error("Command timed out after {timeout:?}")]
    #[diagnostic(code(changeguard::process::timeout))]
    Timeout { timeout: Duration },

    #[error("Process exited with status {status}")]
    #[diagnostic(help("Check the captured stderr for more details."))]
    Failed { status: i32, stderr: String },
    
    #[error("I/O error during subprocess execution")]
    IoError(#[from] std::io::Error),
}

pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub truncated: bool,
}

pub struct CommandOptions {
    pub timeout: Duration,
    pub max_output_bytes: usize,
}
```

## 4. Resilience and Testing
- **Hung Processes**: Must not block the main CLI thread indefinitely.
- **Testing**: Tests must include artificial delays (`sleep` or `Start-Sleep`) to verify timeout triggering and process termination.
- All tests for this module must be runnable safely via `cargo test -j 1` to prevent race conditions during process spawning and timing assertions.
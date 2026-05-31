# Specification: Master Remediation & Hardening (Track R-B2)

## Overview
Address the high-severity findings from the Master Codex Review to achieve true production reliability and architectural cleanliness.

## Requirements
1. **Real Process Timeouts**: In `client_cli.rs`, use `child.kill()` if the subprocess doesn't finish within the timeout period.
2. **Leak-Proof IPC**:
   - Refactor `IpcClient` to avoid detached thread leaks. Use a shared background worker or non-blocking I/O.
   - Implement read timeouts for IPC responses.
3. **Strict Schema Validation**:
   - Make `deserialize_record` return an error if the `version` field is missing or doesn't match `0.2`.
4. **Clean Context Blending in Ask**:
   - Refactor `src/commands/ask.rs` to deduplicate insights/ADRs between the packet summary and the prompt injection.
   - Ensure Gemini truncation handles the *entire* context block including bridge insights.
5. **IPC Query Implementation**:
   - Implement the actual query/response protocol over the named pipe in `client.rs`.

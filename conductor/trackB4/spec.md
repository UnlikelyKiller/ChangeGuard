# Specification: Bridge Query Client (Track B4)

## Overview
Allows ChangeGuard to actively recall information from AI-Brains using a shell execution fallback when IPC is unavailable.

## Architecture & SRP
- **CLI Layer**: `src/commands/bridge.rs` (subcommand `query`)
- **Logic Layer**: `src/bridge/client.rs`
- **Responsibility**: Translate the user query string to an `ai-brains recall` shell command and serialize the result.

## Requirements
- Support `changeguard bridge query <query>`.
- Invoke `ai-brains recall "<query>" --format ndjson` using `std::process::Command`.
- Capture standard output, parse as `BridgeRecord::Insight`, and display or return to caller.
- **Fail-open**: If the `ai-brains` executable is missing from `PATH` or the command errors, emit a warning via `tracing::warn!` and return an empty `Vec<Insight>`. Do not halt execution.

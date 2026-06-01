# Track U2 Spec: AI-Brains Daemon Status Subcommand

## Background
Currently, the AI-Brains background service / daemon can be managed through the `ai-brains daemon` command (supporting subcommands like `start`, `stop`, `schedule`, `unschedule`), but there is no `status` subcommand to inspect if the daemon is currently running, bound to its ports, or what process ID it is using.

## Objective
Implement a `status` subcommand for the `ai-brains daemon` CLI to provide complete operational visibility of the background service.

## Proposed Design
* Register `status` subcommand inside `crates/ai-brains-cli/src/commands/daemon.rs`.
* Check if port `8083` (embeddings) and `8081` (completions) are actively listening or pingable.
* Check if a daemon PID file (e.g. `ai-brainsd.pid`) is present and verify if the process is active using native OS signals/APIs.
* Output structured state (JSON or human-readable format) showing PID, uptime, and port connectivity.

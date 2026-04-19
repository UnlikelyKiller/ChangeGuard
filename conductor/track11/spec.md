# Specification: Track 11 - Ask Gemini Baseline

## Overview
Implement the `ask` command to provide change intelligence assisted by Gemini. The command will load the latest `ImpactPacket`, construct a contextual prompt, and pipe it into the `gemini` CLI tool.

## Components

### Prompt Builder (`src/gemini/prompt.rs`)
- `pub fn build_system_prompt() -> String`
- `pub fn build_user_prompt(packet: &ImpactPacket, query: &str) -> String`
- Injects:
    - Current repository context (branch, head).
    - Summary of changes.
    - Risk level and reasons.
    - Extracted symbols.

### Gemini Runner (`src/gemini/mod.rs`)
- `pub fn run_query(full_prompt: &str) -> Result<()>`
- Executes `gemini` subprocess using the `ExecutionBoundary`.
- Streams output directly to the user's terminal if possible, or captures and prints.

### Ask Command (`src/commands/ask.rs`)
- `pub fn execute_ask(query: String) -> Result<()>`
- Orchestrates loading state, building prompt, and running query.

## Integration
- Use `ExecutionBoundary` for subprocess control.
- Ensure `gemini` executable is detected (via `doctor` logic).

## Verification
- Unit tests for prompt construction.
- Integration tests in `tests/cli_ask.rs` (mocking the `gemini` command).

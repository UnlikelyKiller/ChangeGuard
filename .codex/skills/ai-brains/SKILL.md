---
name: ai-brains
description: "Persistent memory and project context vault. Use this skill whenever the user asks 'what did we decide', mentions past sessions, or when starting work on a repo cold. Trigger when you hear 'remember this', 'don't forget', 'check the vault', or 'what did we decide about'. ALSO trigger on frustration signals like 'I told you last time' or 'we already tried that'. Use even if memory isn't explicitly mentioned if the task involves project history. DO NOT use for generic coding questions, library documentation, or formatting help."
---

# AI-Brains Memory Protocol

This skill provides access to the long-term memory vault. Use it to avoid repeating work and to stay aligned with established architectural decisions.

## When NOT to use this skill
- **Generic Knowledge**: Do not use for general "How to" questions (e.g., "How do I use unwrap in Rust?").
- **Trivial Edits**: Do not use for one-off formatting or simple syntax fixes.
- **Immediate Context**: Do not use if the answer is already visible in the current session's immediate conversation history.

## Availability & Fallback
This skill requires the `ai-brains` CLI tool.
1. **Check**: Run `ai-brains --version`. If it prints usage info, the CLI is available.
2. **Fallback**: If the CLI is not found, inform the user that ai-brains needs to be installed. Proceed with manual context gathering (README, Cargo.toml, entry points) and do not attempt further vault commands.

## Infrastructure Invariants (May 2026)
- **Daemon Auto-Launch**: The CLI automatically spawns `ai-brainsd` in the background if it's unreachable. It inherits vault path and key from the environment.
- **Ultra-Fast Handshake**: The CLI performs an async Ping/Pong handshake with the daemon in **<10ms**.
- **Fast-Fail**: Daemon-dependent commands return `exit 1` immediately if the daemon is unreachable and auto-launch fails.
- **Structured Errors**: All CLI failures emit structured JSON objects (`ApiResult::error`) to stderr.

## Workflow Phases

### Phase 1: Orient (What do I already know?)
Trigger when starting a new session or entering a new repository.
1. **Sync Safety**: Run `ai-brains safety sync`.
   - **Goal**: Ingest recent ChangeGuard hotspots to identify brittle files.
   - **Tip**: Use `--dry-run` to preview what would be synced without pinning.
2. **Get Orientation**: Run `ai-brains preflight --max-words 1000`.
   - **Goal**: Identify the most recent project state and safety constraints.
   - **Tip**: Use `--pretty` for human-readable text output when debugging.
- **Heuristic**: Keep any additional manual research notes under ~150 words to ensure the memory index remains dominant in your context.

### Phase 2: Recall (Search before acting)
Trigger before starting a development track, architectural change, or when an unfamiliar constant/path is encountered.
1. **Unified Search**: Run `ai-brains sync query "<topic>" --quiet`
   - **Goal**: Search both local vault and ChangeGuard bridge records in one command.
   - **Tip**: Use `--quiet` to suppress ChangeGuard bridge noise (e.g., file locks).
2. **Vault Search**: Run `ai-brains recall "<topic>"`
   - **Goal**: Find project-specific constraints or rejected approaches in the local vault.
   - **Context**: This command traverses FTS5 with BM25 ranking.
   - **Readable output**: Use `--format pretty` for human-readable results with scores displayed.

### Phase 3: Record (Persist after deciding)
Trigger immediately after a major decision, discovery of a critical constraint, or user correction.
Run: `ai-brains pin "DECISION: <content>"`
- **Goal**: Pin "Dense" knowledge (decisions, invariants, constraints).
- **Format**: Use the format `DECISION: ...`, `CONSTRAINT: ...`, or `INVARIANT: ...`.
- **Role Selection**: Use the default (assistant) when recording your own reasoning. Use `--role user` when recording a direct correction or instruction from the user.
- **Tags**: Use `--tag <tag>` (repeatable) to categorize memories (e.g., `--tag architecture --tag database`).
- **Stdin**: Use `--stdin` to pipe long content instead of a positional argument.

### Phase 4: Forget (Correct mistakes)
Trigger when a memory is wrong, outdated, or was created for testing.
- **By ID**: `ai-brains forget --memory-id <uuid> -f` — forgets a specific memory (use `-f` to skip confirmation).
- **By content**: `ai-brains forget --match "<search terms>" -f` — finds and forgets by content match.
- **List**: `ai-brains forget --list-forgotten` — shows all forgotten memories.
- **Restore**: `ai-brains forget --restore <uuid>` — un-forgets a memory via compensating event.

## Integration & Automation

### Antigravity (`agy`) CLI
The system supports the new `agy` CLI via real-time hooks and multi-path discovery.
- **agy-hook**: Triggered by `agy` to push turns into the vault. Enforces privacy filtering (user/assistant only).
- **Expanded Import**: `ai-brains antigravity-import` scans tool-specific brain dirs and project-specific tmp chat folders (`session-*.jsonl`).

## Maintenance
For batch reconciliation across sessions and to update the relational graph, run:
`ai-brains nightly`
- **Graceful Management**: Use `ai-brains daemon stop` to shutdown the background process before upgrades. Use `--force` if it hangs.
- **Scheduling**: Use `--schedule` to register as a Windows scheduled task. Use `--unschedule` to remove it.

## Backup & Restore
- **Create backup**: `ai-brains backup` (or `ai-brains backup create --output-dir <path>`)
- **Restore**: `ai-brains backup restore <path>` — verifies integrity before restoring, prompts for confirmation.

## Command Summary

| Action | Command |
|---|---|
| Initialize Context | `ai-brains context` (use `--show` to view, `--new-session` reset) |
| Sync Safety Signals | `ai-brains safety sync` (use `--dry-run` to preview) |
| Unified Search | `ai-brains sync query` (searches vault + ChangeGuard) |
| Get Orientation | `ai-brains preflight` (use `--pretty` for readable text) |
| Deep Search | `ai-brains recall` (use `--format pretty` for readable results) |
| Pinned Record | `ai-brains pin` (use `--tag` for categories, `--stdin` piped) |
| Forget Memory | `ai-brains forget` (use `--match` for search, `--restore` undo) |
| agy Capture Hook | `ai-brains agy-hook --payload "{...}"` (used by agy CLI hooks) |
| Import Antigravity | `ai-brains antigravity-import --days 30` (scans multi-path brain/tmp) |
| Nightly Sweep | `ai-brains nightly` (summarization + graph rebuild) |
| Sync Pull/Push | `ai-brains sync pull`, `ai-brains sync push` (interchange with bridge) |
| Stop Daemon | `ai-brains daemon stop` (use `--force` to kill process) |
| Backup Vault | `ai-brains backup` (use `backup restore <path>` to recover) |


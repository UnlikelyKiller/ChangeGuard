# Track Y4: Progress Feedback for Blocking Operations

**Status:** Planned  
**Milestone:** Y — CLI Reliability & UX Hardening  
**Priority:** Medium

## Objective

Add progress indicators to blocking CLI operations so users see activity during long waits. Currently `changeguard ask` silently waits up to 15s, `changeguard verify` runs commands with no progress until they complete, and `changeguard index --semantic` processes local model inference without any status line. Users reasonably think these commands have hung.

## Problem Statement

Three operations trigger silent waiting:

1. **`changeguard ask`** — queries local LLM with a default 15s per-request timeout. No output is emitted between sending the query and receiving the response. A 15s pause with no feedback feels like the command hung.

2. **`changeguard verify <command>`** — runs the configured test command. Depending on test suite size, this can take 30s+. No output is shown until the command exits.

3. **`changeguard index --semantic`** — processes embeddings through local model inference. No progress lines beyond log-level traces.

4. **Stale-index prompt** — When the semantic index is stale, `changeguard ask --semantic` uses `inquire::Confirm` which blocks scripts and CI. No `CHANGEGUARD_NON_INTERACTIVE` env-var guard exists.

## Acceptance Criteria

1. Before each blocking LLM call in `ask`, print a brief status line (e.g., `"Contacting LLM..."`) that is cleared or followed by the response.
2. Before `verify` runs a command, print `"Running: <command>..."` that is replaced by the result.
3. Before `index --semantic` processing, print the number of files to embed and update a counter as they complete.
4. All progress output goes to stderr, not stdout, so `--json` or piped output is not contaminated.
5. Spinner/progress is suppressed when `--json` is active or when `CHANGEGUARD_NON_INTERACTIVE` is set.
6. `CHANGEGUARD_NON_INTERACTIVE` env-var gate skips the `inquire::Confirm` prompt and falls back to `--auto-index` behavior.

## API Contracts

No new CLI flags. Progress output goes to stderr. `CHANGEGUARD_NON_INTERACTIVE` env var (any non-empty value = non-interactive).

## Key Files

- `src/commands/ask.rs` — LLM call site, staleness prompt
- `src/commands/verify.rs` — command execution
- `src/commands/index.rs` — semantic indexing progress
- `src/index/staleness.rs` — stale-index prompt

## Definition of Done

- `changeguard ask "..."` prints a brief message before contacting the LLM.
- `changeguard verify` prints `"Running: <command>..."` before execution.
- `changeguard index --semantic` shows per-file progress.
- `changeguard ask --json "..."` suppresses progress output.
- `CHANGEGUARD_NON_INTERACTIVE=true changeguard ask --semantic "..."` skips the interactive prompt.
- All existing tests pass.
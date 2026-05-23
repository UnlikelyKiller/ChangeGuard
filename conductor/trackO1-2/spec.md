# Track O1-2: Intent Capture LLM Pipeline

## Objective
Implement the local LLM generation pipeline for drafting the structured intent payload from git diffs and commit messages.

## Requirements
*   Define a JSON grammar constraint (or structured prompt) for Gemma 4 to output `IntentState`.
*   Assemble a context window consisting of:
    *   Unified diff (truncated to 2000 tokens).
    *   `.git/COMMIT_EDITMSG` if present.
    *   Recent ledger history from the current branch.
*   Integrate with the existing `LocalModelClient`.
*   Define confidence calibration: Map model token logprobs or semantic heuristic to a `0.0 - 1.0` confidence score.

## Fallback Behavior
*   Enforce a strict latency budget (2 seconds). If the local model times out or is unreachable, the pipeline must immediately fall back to an empty TUI without blocking the developer's workflow.
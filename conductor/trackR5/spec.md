# Specification: Context-Aware Intelligence Defaults

## Objective
Enhance the user experience of the `ask` command. If the user invokes `changeguard ask "question"` but there are no staged or dirty changes, automatically default to searching the global graph context instead of immediately erroring.

## Requirements
- Target file: `src/commands/ask.rs` (and potentially `src/gemini/modes.rs`).
- Currently, `ask` errors with "No changes to analyze" if local diffs are empty.
- Change logic: If Diff is empty, print an info message (e.g., "No staged changes detected. Defaulting to global repository context.") and switch context mode to `GLOBAL`.
- Still support explicit `--mode` overrides if the user forces diff mode and it fails.

## Architecture
- Graceful degradation in `AskCommand::execute`.
- Before sending prompt to Gemini/Local Model, check the diff payload size. If 0 and mode is `AUTO` or `DIFF` (without strict force), swap mode to `GLOBAL`.
# Track U4 Spec: Risk De-Noising (Ignore Logic Refinement)

## Background
Currently, `filter_ignored_changes` in `src/git/ignore.rs` only excludes **untracked** files (unstaged Added). Tracked files that match `watch.ignore_patterns` (such as `.agents/`, `.claude`, `.codex`) are still included in `scan --impact`. This leads to noisy "High Risk" ratings due to high temporal coupling between metadata files, even when no logic has changed.

## Objective
Refine the ignore logic to ensure that any file matching `watch.ignore_patterns` is excluded from the **Impact Analysis** pipeline, regardless of whether it is tracked by Git or not.

## Proposed Design
* Update `filter_ignored_changes` in `src/git/ignore.rs` to optionally filter all files, not just untracked ones.
* Specifically target high-noise metadata files like `.agents/`, `.claude`, and `.codex`.
* Update `execute_scan` in `src/commands/scan.rs` to pass a flag indicating that all ignored files should be stripped before passing the change set to the `ImpactOrchestrator`.
* Ensure that the `scan` summary (without `--impact`) still shows tracked dirty files, but perhaps with a "Hidden from Impact" annotation or simply filter them out entirely if they match the ignore list to keep the UI clean.

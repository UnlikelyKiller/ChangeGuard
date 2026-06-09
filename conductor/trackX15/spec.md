# Track X15: `watch` Mode Stability and Exit Handling

**Status:** Planned  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

`changeguard watch` starts a file-watcher that re-runs impact analysis on changes. The command works but has two friction points: (1) it does not show what it is watching before printing events, leaving users wondering if it started; (2) Ctrl+C on Windows sometimes exits with a non-zero code rather than a graceful `0`, which confuses CI wrappers that invoke watch in a subshell.

## Problem Statement

1. After `changeguard watch` starts, there is a delay before any output — the user sees a blank terminal and may think it hung.
2. On Windows, `ctrlc` signal handling may not reset the exit code to 0 before process exit, causing tools like PowerShell to report a failure.

## Acceptance Criteria

1. Immediately after starting, `watch` prints:
   ```
   Watching: C:\dev\ChangeGuard  (press Ctrl+C to stop)
   ```
   where the path is the resolved repo root.

2. On Ctrl+C, the process exits with code 0 and prints:
   ```
   Watch stopped.
   ```

3. The watcher respects `.gitignore` and `.changeguard/ignore` patterns — changes to `.changeguard/state/` do not trigger re-analysis.

4. No changes to watch event logic — startup and exit handling only.

## Key Files

- `src/commands/watch.rs` — `execute_watch`
- `src/cli.rs` — `WatchArgs`

## Definition of Done

- Starting `changeguard watch` immediately shows the watching path.
- Ctrl+C exits with code 0 and prints "Watch stopped."
- `.changeguard/state/` changes do not trigger re-runs.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

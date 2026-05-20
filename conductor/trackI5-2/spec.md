# Track I5-2: Fix Scan Command to Respect Ignore Patterns

## Status
In Progress

## Issue
`changeguard scan` flags all dirty files from `get_repo_status()` without filtering against `config.watch.ignore_patterns`. This causes agent dotfiles (`.claude/`, `.codex/`, `.agents/`, `.opencode/`) and other ignored paths to appear in scan output.

## Root Cause
`execute_scan()` in `src/commands/scan.rs` never loads the config or filters the changes from `get_repo_status()`.

## Fix
Load config in `execute_scan()` and filter the changes using `globset::GlobSet` matching against `config.watch.ignore_patterns`.

## Test Plan
1. `cargo test --workspace` — existing tests pass
2. `changeguard scan` — dotfile paths no longer appear in output

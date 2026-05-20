# Track I5-3: Fix viz --output Path Handling

## Status
Completed

## Issue
`changeguard viz --output /path/to/output.html` fails silently or creates output in the wrong location when the parent directory of the specified output path doesn't exist.

## Root Cause
`execute_viz()` in `src/commands/viz.rs` calls `fs::write(&out, html)` which only creates the file but not parent directories. If the parent directory doesn't exist, the write fails.

## Fix
Ensure parent directory exists before writing using `fs::create_dir_all()` on the parent of the output path.

## Test Plan
1. `cargo test --workspace` — existing tests pass
2. `changeguard viz --output C:\temp\custom\graph.html` — creates custom dir and file

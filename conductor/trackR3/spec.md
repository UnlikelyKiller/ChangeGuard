# Specification: Proactive Index Repair & Health

## Objective
Provide actionable insights into the health of the codebase indices via the `doctor` command, catching corruption or severe staleness before they impact user workflows.

## Requirements
- Extend `src/commands/doctor.rs`.
- **Tantivy Index Integrity**: Check if `.changeguard/state/tantivy` directory is accessible and cleanly opens, checking for known corruption markers.
- **CozoDB Staleness**: Check graph staleness (e.g., comparing latest commit hash in graph vs actual HEAD).
- **Repair Suggestions**: If health checks fail, output explicit actionable commands like `changeguard index --full`.

## Architecture
- Call into `src/index/staleness.rs` and `src/search/tantivy_engine.rs` (or similar modules) to probe index state without triggering a full load.
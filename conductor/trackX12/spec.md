# Track X12: `hotspots explain` Filters Directory-Level Temporal Coupling Noise

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

`changeguard hotspots explain <file>` includes directory-level entries (e.g., `src/`, `src/index/`) in the Top Couplings list. These are noise — directories are not files and their coupling scores conflate many different relationships. Only file-level entries (paths with extensions) should appear in the coupling display.

## Problem Statement

`TemporalEngine::calculate_couplings()` returns entries where `file_a` and `file_b` can be either files or directories. The `execute_hotspots_explain` function filters by matching against the entity path but does not exclude entries where the *other* side of the coupling is a directory path (no extension, ends in `/` or `\`, etc.).

Example noisy entry: `src\index\` with score 0.72 — this is the parent directory, not a specific file.

## Acceptance Criteria

1. The Top Couplings display in `hotspots explain` only shows entries where the coupled partner has a file extension (i.e., is a file, not a directory).
2. Entries where either `file_a` or `file_b` is a directory are silently excluded.
3. The `Temporal Couplings: N` count shows the filtered count (files only), not the raw count.
4. A note is added when entries are filtered: `"(N directory-level entries hidden)"` in dim text, only when N > 0.

## Key Files

- `src/commands/hotspots.rs` — `execute_hotspots_explain` (lines 259–279)

## Definition of Done

- `hotspots explain src/commands/hotspots.rs` shows only file-level coupling partners (e.g., `src/commands/ledger.rs`) not directory entries.
- The filtered count note appears when directories were excluded.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

# Track U20 Spec: Always-Visible Semantic Index Lifecycle Logging

## Background

After U14 added the `info!("Semantic indexing threads: parse=N, embed_concurrency=M")` log line, I observed during smoke testing that the line **does not fire on incremental runs where nothing has changed** — the early-exit at `src/commands/index.rs:612` (`if files_to_process.is_empty()`) returns *before* the new code runs.

This makes U14's new resolver look like a no-op to anyone running `changeguard index --semantic --incremental` on a clean repo. The fix is structural: move the lifecycle logs to fire *before* any early-exit, so users always see them when they invoke `--semantic`.

A related gap: the "Semantic index is up to date" message uses `println!` (stdout contract noise), but should be a structured `info!` event so it's suppressible, redirectable, and machine-parseable. The 2026 Rust best practice (perl-lsp, spotifai migrations) is: **stdout = machine contract, stderr = human, tracing = everything structured**.

## Objective

Restructure the `execute_semantic_index` function so that the lifecycle events (thread resolution, batch planning, completion) all log via `tracing::info!` *before* the early-exit, and the "up to date" message also uses `tracing::info!` (not `println!`). The function should be observably identical for the user — same messages, same exit codes — but the messages should be in the right stream and in the right place.

## Proposed Design

### Function restructure

In `src/commands/index.rs::execute_semantic_index`:

1. **Phase 0: Lifecycle header** (new, always runs)
   ```rust
   info!("Semantic indexing started: incremental={incremental}, cli_concurrency={:?}", concurrency_override);
   let resolved = resolve_semantic_concurrency(...);
   info!("Semantic indexing threads: parse={parse_threads}, embed_concurrency={embed_cap}");
   ```

2. **Phase 1: File walk** (unchanged)

3. **Phase 1.5: Early-exit with structured log** (changed)
   ```rust
   if files_to_process.is_empty() {
       info!("Semantic index is up to date: no files changed since last index");
       return Ok(());
   }
   info!("Semantic indexing will process {} files", files_to_process.len());
   ```

4. **Phases 2-4: existing work, no change**

### Tracing best practices

Per 2026 consensus (perl-lsp PR #3245, spotifai commit `648ab51`):
- `info!` for lifecycle (start, end, phase boundaries)
- `debug!` for routine per-file operations
- `trace!` for hot-path details
- `println!` reserved for the stdout contract (the final summary line `Semantic indexing complete: N/N files produced embeddings.`)

### Test strategy

The change is mostly mechanical, so the test is behavioral:
- Capture `tracing` output via a `tracing_subscriber::fmt::TestWriter` and assert the lifecycle log fires *before* the "up to date" message
- Confirm that no `println!` call happens for the "up to date" case (only the structured info event)

## Critical files

| File | Change |
|---|---|
| `src/commands/index.rs` | Move Phase 2 log to before the empty check; switch "up to date" from `println!` to `info!`; add `Semantic indexing will process N files` between early-exit and Phase 2 |
| `src/main.rs` | No change (tracing-subscriber already installed) |

## Existing utilities to reuse

- `tracing::info!`, `tracing::debug!` (already used throughout the codebase)
- `tracing_subscriber::EnvFilter` (installed in `main.rs` per J1's quiet-by-default filter)

## TDD plan (Red → Green)

1. `src/commands/index.rs` test:
   - `lifecycle_log_fires_on_empty_index`: capture `tracing` events, run `execute_semantic_index` on a clean repo, assert `Semantic indexing started:`, `Semantic indexing threads:`, and `Semantic index is up to date:` all fire *in order* with no `println!` in between for the up-to-date case
   - `no_println_on_up_to_date_path`: capture stdout/stderr separately, assert "up to date" appears on stderr (via tracing) not stdout

2. Implementation: reorder the log lines, switch the `println!` to `info!`.

## Verification

1. CI gate.
2. Manual: on a clean repo (no source changes), `changeguard index --semantic --incremental` should log the thread-resolution line and the "up to date" message, both via tracing.
3. `RUST_LOG=info cargo run -- index --semantic --incremental` shows all three lines; `RUST_LOG=warn` suppresses them.
4. `changeguard index --semantic --incremental 2>/dev/null` should produce no "up to date" output on stdout (it's now on stderr via tracing).

## Why this scope

This is **opportunity #2 from the U14 retrospective** — the most-visible UX miss. U15's `--semantic-dry-run` would also fix this (the dry-run mode doesn't early-exit because it always runs the resolver), but the lifecycle log is the right *baseline* fix: it should always have worked. The dry-run is an additional capability, not a replacement for visible lifecycle events.

## Out of scope

- Restructuring all `println!` calls in the codebase (would need a separate audit; deferred to a future track)
- Adding a `--quiet` flag to suppress lifecycle events (the `RUST_LOG=warn` escape hatch is sufficient)
- Reworking the success message `Semantic indexing complete: 366/366 files produced embeddings.` (it stays on stdout — it's the binary's contract)

## References

- U14 lifecycle log: `src/commands/index.rs:639` (currently after the early-exit)
- 2026 tracing best practices: perl-lsp PR #3245, spotifai `648ab51`, normalize `210999d`
- J1 INFO→DEBUG migration: the reason the "Indexing repository for semantic search..." log line at line 545 may also be downgraded — verify it's still `info!` in U20 scope

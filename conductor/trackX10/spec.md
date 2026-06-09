# Track X10: `ledger gc` No-Args UX and `ledger adr list`

**Status:** Planned  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

Two `ledger` subcommand UX issues:
1. `ledger gc` with no mode flag (neither `--stale` nor `--orphans`) gives a raw validation error: `"error: provide --stale or --orphans"`. This should be a friendly usage hint.
2. `ledger adr list` is implied by the `ledger adr` command but may not exist or may require additional flags that are not obvious.

## Problem Statement

- `ledger gc` requires a mode but clap validates before any user-facing code runs, so the error message is low-level clap output rather than a ChangeGuard-styled hint.
- `ledger adr` shows a subcommand menu but `list` was not available as a standalone command prior to U27 cleanup. Users may try `ledger adr list` expecting a ledger-formatted ADR list.

## Acceptance Criteria

1. `changeguard ledger gc` (no args) prints:
   ```
   Usage: changeguard ledger gc --stale [--ttl-hours <N>] | --orphans [--force]
   
   Modes:
     --stale     Remove PENDING transactions older than TTL (default: 72h)
     --orphans   Remove transactions with no corresponding git commit
   ```
   Styled with cyan command name, then exits with code 0.

2. `changeguard ledger adr list` works and shows all ADR entries from the ledger in a table: `[ID, Entity, Status, Title, Created]`.

3. `changeguard ledger adr` (no subcommand) also shows the list (backwards compat, already may work).

## Key Files

- `src/commands/ledger.rs` — `execute_ledger_gc` (no-args guard)
- `src/commands/ledger_adr.rs` — `execute_ledger_adr`
- `src/cli.rs` — CLI subcommand definition for `LedgerAdrSubcommand`

## Definition of Done

- `changeguard ledger gc` with no args prints styled usage and exits 0.
- `changeguard ledger adr list` shows ADR rows or "No ADRs found" hint.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

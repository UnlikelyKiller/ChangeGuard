# Track X10 Plan: `ledger gc` UX and `ledger adr list`

## Phase 1 — `ledger gc` No-Args Guard
- [x] 1. In `src/commands/ledger.rs` `execute_ledger_gc`, at the top of the function, check if neither `--stale` nor `--orphans` was set:
  ```rust
  if !args.stale && !args.orphans {
      println!("{}", "Usage: changeguard ledger gc --stale [--ttl-hours <N>] | --orphans [--force]".cyan());
      println!();
      println!("  {}  Remove PENDING transactions older than TTL (default: 72h)", "--stale".bold());
      println!("  {}  Remove transactions with no corresponding git commit", "--orphans".bold());
      return Ok(());
  }
  ```
- [x] 2. Verify `args.stale` and `args.orphans` are `bool` fields (from existing LedgerGcArgs). If they're required mode flags already, adjust accordingly.

## Phase 2 — `ledger adr list`
- [x] 3. In `src/commands/ledger_adr.rs`, check if `LedgerAdrSubcommand::List` variant exists. If not, add it to the CLI in `src/cli.rs`.
- [x] 4. Implement `execute_ledger_adr_list(storage) -> Result<()>`:
  - Query SQLite: `SELECT id, entity, status, title, created_at FROM ledger_adrs ORDER BY created_at DESC`.
  - Render as a `comfy-table` table with headers `[ID, Entity, Status, Title, Created]`.
  - Empty state: `"No ADRs found. Start a ledger transaction with 'changeguard ledger start <entity>'.".yellow()`.
- [x] 5. Wire `LedgerAdrSubcommand::List` to `execute_ledger_adr_list` in `execute_ledger_adr`.

## Phase 3 — Verification
- [x] 6. Run `changeguard ledger gc` (no args) — confirm styled usage printed, exit 0.
- [x] 7. Run `changeguard ledger adr list` — confirm table shown (or empty hint).
- [x] 8. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 9. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 10. Run `cargo fmt --all -- --check` — clean.
- [x] 11. Update `conductor/conductor.md` status to Completed.

# Track X10 Plan: `ledger gc` UX and `ledger adr list`

## Phase 1 — `ledger gc` No-Args Guard
- [ ] 1. In `src/commands/ledger.rs` `execute_ledger_gc`, at the top of the function, check if neither `--stale` nor `--orphans` was set:
  ```rust
  if !args.stale && !args.orphans {
      println!("{}", "Usage: changeguard ledger gc --stale [--ttl-hours <N>] | --orphans [--force]".cyan());
      println!();
      println!("  {}  Remove PENDING transactions older than TTL (default: 72h)", "--stale".bold());
      println!("  {}  Remove transactions with no corresponding git commit", "--orphans".bold());
      return Ok(());
  }
  ```
- [ ] 2. Verify `args.stale` and `args.orphans` are `bool` fields (from existing LedgerGcArgs). If they're required mode flags already, adjust accordingly.

## Phase 2 — `ledger adr list`
- [ ] 3. In `src/commands/ledger_adr.rs`, check if `LedgerAdrSubcommand::List` variant exists. If not, add it to the CLI in `src/cli.rs`.
- [ ] 4. Implement `execute_ledger_adr_list(storage) -> Result<()>`:
  - Query SQLite: `SELECT id, entity, status, title, created_at FROM ledger_adrs ORDER BY created_at DESC`.
  - Render as a `comfy-table` table with headers `[ID, Entity, Status, Title, Created]`.
  - Empty state: `"No ADRs found. Start a ledger transaction with 'changeguard ledger start <entity>'.".yellow()`.
- [ ] 5. Wire `LedgerAdrSubcommand::List` to `execute_ledger_adr_list` in `execute_ledger_adr`.

## Phase 3 — Verification
- [ ] 6. Run `changeguard ledger gc` (no args) — confirm styled usage printed, exit 0.
- [ ] 7. Run `changeguard ledger adr list` — confirm table shown (or empty hint).
- [ ] 8. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 9. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 10. Run `cargo fmt --all -- --check` — clean.
- [ ] 11. Update `conductor/conductor.md` status to Completed.

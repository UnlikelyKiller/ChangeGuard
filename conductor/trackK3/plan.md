# Track K3: CLI UX Polish Plan

## Phase 1: Aliases & Routing
- [ ] Add `Status` subcommand to `Cli` top-level enum in `src/cli.rs`.
- [ ] Map `status` to `execute_ledger_status` (refactor from `ledger.rs`).
- [ ] Add `upgrade` as an alias for `update` in `src/cli.rs`.

## Phase 2: Actionable Error Diagnostics
- [ ] Refactor `StateError` to include a `miette` help message for `SchemaMismatch`.
- [ ] Update `execute_search` and `execute_ask` to handle schema errors gracefully.
- [ ] Verify help text appears in terminal on artificial schema break.

## Phase 3: Search Intelligence
- [ ] Implement regex detection heuristic in `src/search/mod.rs`.
- [ ] Update `execute_search` to switch modes based on heuristic if no flag is provided.
- [ ] Add Mode Header to search output.
- [ ] (Optional) Add `--hybrid` flag implementation.

## Phase 4: Verification
- [ ] Manual check: `changeguard status`.
- [ ] Manual check: `changeguard search "^pub struct"` (no `-r` flag).
- [ ] CI Gate.

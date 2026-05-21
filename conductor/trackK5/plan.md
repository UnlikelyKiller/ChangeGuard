# Track K5: Operational Transparency Plan

## Phase 1: Configuration Visibility
- [ ] Implement `execute_config_view` in `src/commands/config.rs`.
- [ ] Update `src/cli.rs` to wire the `view` subcommand.
- [ ] Add logic to `Config` loading to track source provenance of fields (optional but high value).
- [ ] Verify output with `changeguard config view`.

## Phase 2: Audit Pagination
- [ ] Update `LedgerDb::get_audit_log` (or equivalent) in `src/ledger/db.rs` to accept limit and offset.
- [ ] Add `limit` and `offset` fields to `AuditArgs` in `src/cli.rs`.
- [ ] Implement pagination in `execute_audit` in `src/commands/ledger.rs`.
- [ ] Verify with manual commands and a new integration test.

## Phase 3: Final Verification
- [ ] Run full CI gate.
- [ ] Verify `config view --json` is valid.

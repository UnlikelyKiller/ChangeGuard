# Track W12 Plan: Ledger Transaction Entity Links and Validator UX

- [x] Task W12.1: Add failing CLI tests for validator list, disable, enable, remove, and doctor flows.
- [x] Task W12.2: Add migrations for validator IDs and transaction entity links.
- [x] Task W12.3: Implement validator lifecycle commands with non-interactive output.
- [x] Task W12.4: Link ledger transactions to W1 graph entities during start, commit, adoption, rollback, and hook promotion.
- [x] Task W12.5: Implement `ledger graph <tx-id>` with human and JSON output.
- [x] Task W12.6: Add hook lifecycle diagnostics and repair commands for sidecar/pending mismatches.
- [x] Task W12.7: Add stable provenance export schema.
- [x] Task W12.8: Run ledger lifecycle, hook, validator, integration, and full verification gates; reinstall.

## Definition of Done Checklist

- [x] Validator UX does not require manual database edits.
- [x] Transaction graph output shows affected entities and evidence.
- [x] Hook mismatch repair is explicit and auditable.
- [x] Full verification gate passes.

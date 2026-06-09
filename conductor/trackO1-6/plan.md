# Plan: Track O1-6 (SOC2 Evidence Export)

- [ ] 1. Define the `Soc2EvidenceRecord` struct representing the auditor-ready schema.
- [ ] 2. Update `src/cli.rs` to include the `changeguard export` top-level command.
- [ ] 3. Create `src/commands/export.rs` to handle parsing `--format` and `--period`/`--since`/`--until` arguments.
- [ ] 4. Implement a ledger query function to retrieve signed entries within the date range.
- [ ] 5. Map the ledger entries to the `Soc2EvidenceRecord` instances.
- [ ] 6. Write the records to `reports/soc2-export-[date].json` (and/or CSV).
- [ ] 7. Write integration tests to verify the command output and formatting.
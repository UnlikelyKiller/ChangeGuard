## Plan: Track B2 - Bridge Export Command
### Phase 1: State Extraction and Export
- [ ] Task 1.1: Add `bridge` subcommand to CLI parser in `src/cli.rs`.
- [ ] Task 1.2: Implement `src/commands/bridge.rs` with the `export` route.
- [ ] Task 1.3: Create `src/bridge/export.rs` to extract hotspots from `latest-impact.json`.
- [ ] Task 1.4: Extract recent ledger commits from CozoDB into `LedgerDelta` records.
- [ ] Task 1.5: Write records line-by-line to the target `--out` path as NDJSON.
- [ ] Task 1.6: Add an integration test verifying the NDJSON file output exactly matches the v0.2 schema contract.

## Plan: Track B3 - Bridge Import Command
### Phase 1: Ingestion and Enrichment
- [x] Task 1.1: Add `import` subcommand to `src/commands/bridge.rs`.
- [x] Task 1.2: Update `ImpactPacket` to support an `ai_insights` array.
- [x] Task 1.3: Create `src/bridge/import.rs` to stream and sequentially parse NDJSON records.
- [x] Task 1.4: Implement logic to rewrite `latest-impact.json` with the newly enriched insights.
- [x] Task 1.5: Write TDD tests ensuring graceful fail-open execution on malformed JSON lines.

# Specification: Bridge Import (Track B3)

## Overview
A command to ingest external AI-Brains memories and enrich the current `ImpactPacket`.

## Architecture & SRP
- **CLI Layer**: `src/commands/bridge.rs` (subcommand `import`)
- **Logic Layer**: `src/bridge/import.rs`
- **Responsibility**: Parse NDJSON streams, extract `BridgeRecord::Insight`, and cleanly append them to the active context.

## Requirements
- Support `changeguard bridge import --in <file.ndjson>`.
- Update `ImpactPacket` struct in `src/impact/model.rs` to include an optional `ai_insights: Vec<BridgeRecord::Insight>` array.
- **Fail-open**: If the file is malformed or an individual line fails to parse, log the error but continue parsing subsequent lines without crashing.

# Specification: Bridge Export (Track B2)

## Overview
Add a command to export ChangeGuard's internal state (hotspots, ledger deltas) to an NDJSON file for batch AI-Brains ingestion.

## Architecture & SRP
- **CLI Layer**: `src/commands/bridge.rs` (subcommand `export`)
- **Logic Layer**: `src/bridge/export.rs`
- **Responsibility**: Query `.changeguard/state/ledger.cozo` and `latest-impact.json`, transform the local structs into external `BridgeRecord`s, and flush to disk.

## Requirements
- Support `changeguard bridge export --out <file.ndjson>`.
- Must map internal ChangeGuard Hotspots to `BridgeRecord::Hotspot`.
- Must read CozoDB ledger transactions and map to `BridgeRecord::LedgerDelta`.

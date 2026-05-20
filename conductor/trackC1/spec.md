# Specification: Contextual Risk Export & Structured MADR Fields (Track C1)

## Overview
Enhance `bridge export` to support targeted scope-based risk analysis and export structured MADR (Markdown Architectural Decision Record) fields for AI-Brains to format and ingest during nightly heartbeat.

## Architecture & SRP
- **Module**: `src/bridge/export.rs`
- **Responsibility**: Export contextual risk data and structured decision fields to AI-Brains via the BridgeRecord IPC channel.

## Requirements
- Extend `export --hotspots` to accept an optional `--scope <paths>` argument (comma-separated file/directory list).
- When scope is provided, calculate cross-repo impacts, temporal coupling scores, and failure risk probabilities specifically for the targeted scope rather than returning global top-N brittle files.
- When scope is omitted, preserve existing behavior (global top-N hotspots).
- Add a `--madr` flag to `bridge export` that emits structured MADR fields (title, context, decision, consequences) as `BridgeRecord` entries from ledger ADR entries.
- MADR export must NOT pre-format markdown — AI-Brains owns the MADR formatting layer. ChangeGuard sends only structured data.
- Results must be deterministic: sort all emitted collections, use stable identifiers.

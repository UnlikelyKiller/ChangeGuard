# Specification: BridgeRecord Data Model (Track B1)

## Overview
Implement the foundational data model for Milestone B: AI-Brains Integration. The `BridgeRecord` struct (v0.2) acts as the strict contract for data shared between ChangeGuard and the external AI-Brains vault.

## Architecture & SRP
- **Module**: `src/bridge/model.rs`
- **Responsibility**: Define pure data structures and serialization rules. Contains no I/O or state manipulation.

## Schema Contracts
All messages must implement `serde::Serialize` and `serde::Deserialize`.
Format is strict NDJSON (Newline Delimited JSON).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", version = "0.2")]
pub enum BridgeRecord {
    Hotspot { path: String, score: f64, reason: String },
    LedgerDelta { tx_id: String, intent: String, files_changed: usize },
    Insight { memory_id: String, relevance: f64, content: String },
    VerifyOutcome { success: bool, command: String, error_snippet: Option<String> }
}
```

## Fail-Open Principles
- Deserialization ignores unknown fields to maintain forwards compatibility.
- Schema version mismatches should log a warning via `tracing::warn!` but attempt best-effort parsing.

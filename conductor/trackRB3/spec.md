# Specification: Track R-B3 — Audit Remediation & Spec Alignment

## Objective
Address all ChangeGuard-related findings from `integration-audit.md` to ensure full compliance with the v0.2 integration spec and seamless interoperability with AI-Brains.

## Requirements

### 1. Spec-Compliant BridgeRecord Schema
- Refactor `BridgeRecord` to match the exact schema specified in `integration.md`.
- Include required metadata: `bridge_version`, `direction`, `timestamp`, `parent_hash`, `project_id`, `session_id`, `tx_id`, `record_kind`, and `privacy`.
- Use a struct-based record with a nested `payload` for variant-specific data.
- Ensure serialization/deserialization is compatible with the AI-Brains implementation.

### 2. IPC Protocol & Framing
- Standardize on **newline-delimited JSON (NDJSON)** for all IPC communication.
- Implement robust multi-record response handling in `IpcClient::receive_records`.
- Add timeout and malformed-frame handling to prevent hangs during IPC.
- Ensure ChangeGuard's newline-delimited output matches AI-Brains' expectations (or standardize both).

### 3. CLI Signature Alignment
- Update `bridge export` to support:
  - `--hotspots`: Selective export of hotspot data.
  - `--ledger`: Selective export of ledger delta data.
  - Both flags are optional; if neither provided, export all (current behavior, but explicit flags are better).
- Update `bridge import` to support:
  - `--from <path>` as the primary flag (spec compatibility).
  - Keep `--in` as a deprecated alias.
  - Support importing ALL record types, not just `Insight`.

### 4. Path & Module Hygiene
- Replace all hardcoded `.changeguard/...` paths in the `bridge` module with `Layout` abstractions.
- Use `camino::Utf8Path` consistently for all internal path handling.

### 5. Provenance & Privacy Realization
- Implement `parent_hash` generation and verification.
- Enforce strictest-wins privacy combining during ingestion.
- Ensure records with `Sealed` or `Private` privacy levels (if applicable) are never exported.

## API Contracts

### BridgeRecord (v0.2)
```rust
struct BridgeRecord {
    bridge_version: String,
    direction: BridgeDirection,
    timestamp: DateTime<Utc>,
    parent_hash: Option<String>,
    project_id: String,
    session_id: Option<String>,
    tx_id: Option<String>,
    record_kind: String,
    payload: serde_json::Value,
    privacy: Privacy,
}
```

## Testing Strategy
- **Unit Tests**: Verify `BridgeRecord` serialization/deserialization against the spec's fixture.
- **Integration Tests**: 
  - Mock AI-Brains IPC server returning multi-line NDJSON and verify ChangeGuard reads all of them.
  - Test selective export with `--hotspots` and `--ledger` flags.
  - Test `bridge import --from` with valid and invalid (lineage-broken) NDJSON.
- **Audit Verification**: Run a final Codex review specifically comparing the implementation against `integration-audit.md`.

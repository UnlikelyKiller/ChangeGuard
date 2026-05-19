## Plan: Track R-B3 — Audit Remediation

### Phase 1: Schema & Path Alignment
- [x] Task 1.1: Refactor `src/bridge/model.rs` to implement the spec-compliant `BridgeRecord` struct and associated enums (`BridgeDirection`, `Privacy`).
- [x] Task 1.2: Update `src/bridge/export.rs` and `src/bridge/import.rs` to use `Layout` abstractions instead of hardcoded paths.
- [x] Task 1.3: Update `src/bridge/model.rs` with `serialize_record` and `deserialize_record` matching the new struct.

### Phase 2: CLI & Feature Gaps
- [x] Task 2.1: Update `src/cli.rs` to add `--hotspots`, `--ledger` to `bridge export` and `--from` to `bridge import`.
- [x] Task 2.2: Implement selective export logic in `src/bridge/export.rs`.
- [x] Task 2.3: Update `src/bridge/import.rs` to handle all `BridgeRecord` types and preserve provenance.

### Phase 3: IPC & Protocol Hardening
- [x] Task 3.1: Refactor `src/bridge/ipc.rs` to handle multi-line NDJSON responses in `receive_records`.
- [x] Task 3.2: Ensure consistent newline termination for all IPC writes.
- [x] Task 3.3: Implement `parent_hash` generation (simple SHA-256 of previous record) and validation.

### Phase 4: Verification & Final Audit
- [x] Task 4.1: Add unit and integration tests for new schema and multi-record IPC.
- [x] Task 4.2: Run `changeguard verify` and ensure no regressions.
- [x] Task 4.3: Perform a final Master Codex Review against `integration-audit.md`.

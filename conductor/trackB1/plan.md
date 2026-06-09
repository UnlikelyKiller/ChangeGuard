## Plan: Track B1 - BridgeRecord Data Model
### Phase 1: Core Models and Serialization
- [x] Task 1.1: Create `src/bridge/mod.rs` and `src/bridge/model.rs`.
- [x] Task 1.2: Define `BridgeRecord` enum and its variants (Hotspot, LedgerDelta, Insight, VerifyOutcome) with v0.2 tags.
- [x] Task 1.3: Write TDD tests for NDJSON serialization ensuring newlines are excluded from individual serialized JSON objects.
- [x] Task 1.4: Write TDD tests for deserializing valid AI-Brains payloads into `BridgeRecord`.

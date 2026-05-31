# Implementation Plan - Track 49: Bound service_map_delta in truncate_for_context()

## Goal
Ensure `service_map_delta` is cleared during context truncation to prevent prompt bloat.

## Proposed Changes

### 1. Clear service_map_delta During Truncation [src/impact/packet.rs]
- In `truncate_for_context()`, find the block that clears enrichment fields (around line 807):
  ```rust
  self.relevant_decisions.clear();
  self.observability.clear();
  self.affected_contracts.clear();
  self.knowledge_graph.clear();
  self.service_map_delta = None;  // ADD THIS LINE
  ```
- This is a one-line addition, consistent with how other large enrichment fields are handled.

### 2. Test [src/impact/packet.rs tests]
- `test_truncate_clears_service_map_delta`:
  - Create a packet with `service_map_delta = Some(ServiceMapDelta { services: vec![...; 50], ... })`.
  - Call `truncate_for_context(100)` (very small budget).
  - Assert `packet.service_map_delta.is_none()`.

## Verification Plan

### Automated Tests
- `cargo test impact::packet::tests`
- `cargo test --workspace`

## Definition of Done (DoD)
- [x] **service_map_delta Cleared**: Truncation clears `service_map_delta` alongside other enrichment fields.
- [x] **Test Coverage**: New test confirms clearing behavior.
- [x] **Zero Regression**: Existing truncation tests pass.
- [x] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

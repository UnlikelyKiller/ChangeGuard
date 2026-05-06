# Track 49: Bound service_map_delta in truncate_for_context()

## Overview
`ImpactPacket::truncate_for_context()` at `src/impact/packet.rs` clears many enrichment fields (relevant_decisions, observability, affected_contracts, knowledge_graph) to stay within context budgets, but `service_map_delta` is left untouched. Large service snapshots — hundreds of services with routes and data models — can blow through the truncation budget and bloat LLM prompts downstream.

## Objectives
- Add `service_map_delta` to the fields cleared or truncated by `truncate_for_context()`.
- Ensure the behavior is consistent with other enrichment fields: clear it when budget is tight.
- Maintain backward compatibility: consumers of truncated packets should already handle `service_map_delta: None` (it uses `#[serde(default)]`).

## Success Criteria
- `truncate_for_context()` sets `service_map_delta = None` alongside other cleared enrichment fields.
- A test with a large `service_map_delta` confirms it is cleared when budget is exceeded.
- Existing truncation tests pass.
- CI gate passes.

## Architecture
- `src/impact/packet.rs` — `truncate_for_context()` method. Add one line clearing `self.service_map_delta`.

## Testing Strategy
- **Red commit**: Test that a packet with a large `service_map_delta` has it cleared after truncation at a small target.
- **Green commit**: Add `self.service_map_delta = None;` at the appropriate position. Verify test passes.

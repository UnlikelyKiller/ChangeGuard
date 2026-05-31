# Track K7: Hotspot API Refactoring (Argument Objects)

## Status
Completed

## Milestone
K: Service Discovery & Storage Hardening

## Problem
The `calculate_hotspots` function has reached 10 arguments, causing Clippy warnings and making the API difficult to use and extend.

## Objective
Refactor the hotspot calculation API to use a structured options object.

## API Changes
- Introduce `HotspotQuery` struct in `src/impact/hotspots.rs`.
- Update `calculate_hotspots` to accept `&HotspotQuery`.

## Success Criteria
- [x] `calculate_hotspots` has fewer than 5 arguments.
- [x] No `#[allow(clippy::too_many_arguments)]` on `calculate_hotspots`.
- [x] All call sites (`commands/hotspots.rs`, `commands/ledger_audit.rs`, `impact/enrichment/hotspots.rs`) updated.
- [x] CI gate passes.

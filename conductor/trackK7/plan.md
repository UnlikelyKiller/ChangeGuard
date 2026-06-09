# Track K7 Plan: Hotspot API Refactoring

## Phase 1: Data Modeling
- [x] Define `HotspotQuery` struct in `src/impact/hotspots.rs`.
- [x] Implement `From<HotspotArgs>` for `HotspotQuery` to bridge CLI args.

## Phase 2: Refactoring
- [x] Update `calculate_hotspots` signature.
- [x] Update call sites in `src/commands/hotspots.rs`.
- [x] Update call sites in `src/commands/ledger_audit.rs`.
- [x] Update call sites in `src/impact/enrichment/hotspots.rs`.
- [x] Update call sites in all test files (`tests/hotspot_ranking.rs`, etc.).

## Phase 3: Final Verification
- [x] Run `cargo clippy` and ensure no `too_many_arguments` warnings.
- [x] Run full CI gate.

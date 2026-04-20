# Track 25 Plan: Hotspot Identification (Risk Density)

## Phase 1: Research & Setup
1. Define the normalization strategy for scaling complexity and frequency to 0-1 ranges.
2. Create a test harness with synthetic history and complexity data.

## Phase 2: Implementation
1. **Engine Scaffolding**:
   - Create `src/commands/hotspots.rs`.
   - Implement `HotspotEngine` to query SQLite for complexity and crawl git for frequency.
2. **CLI Integration**:
   - Add `hotspots` subcommand to `src/main.rs`.
   - Implement argument parsing (limit, format, path filter).
3. **Output Formatting**:
   - Add hotspot table formatting to `src/output/human.rs` and `src/output/table.rs`.
   - Ensure JSON output parity.

## Phase 3: Verification
1. **Unit Tests**:
   - Verify scoring math with controlled inputs.
   - Verify deterministic ranking.
2. **Integration Tests**:
   - Run `changeguard hotspots` on the ChangeGuard repo itself and verify results.

## Phase 4: Review & Merge
1. Review by `@rust-triage-specialist`.
2. Push to `main`.

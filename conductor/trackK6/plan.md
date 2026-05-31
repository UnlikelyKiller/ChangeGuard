# Track K6: Temporal Risk Precision Plan

## Phase 1: Range Filtering
- [ ] Add `commits` and `days` arguments to `HotspotArgs` in `src/cli.rs`.
- [ ] Update `HistoryProvider` (or equivalent) in `src/impact/enrichment/hotspots.rs` to accept range constraints.
- [ ] Implement commit-limited git log traversal.

## Phase 2: Scoring Enhancements
- [ ] Implement exponential decay weighting for older commits in the hotspot algorithm.
- [ ] Add `--since <REF>` support using `gix` to find the merge base.

## Phase 3: Final Verification
- [ ] Compare `hotspots` output with and without range filters on the ChangeGuard repo.
- [ ] Run full CI gate.

# Track I3-2 Plan: Hotspot Score Log-Scaling

## Phase 1 — Red (Failing Tests)

- [ ] Add `normalize_score` stub returning `raw_score` unchanged (no-op).
- [ ] Write unit tests:
  - `log_scale_zero`: assert `normalize_score(0.0) == 0.0`.
  - `log_scale_positive`: assert `normalize_score(1.0) > 0.0`.
  - `log_scale_compresses_outlier`: `normalize_score(0.135) / normalize_score(0.006)` < 5.0 (will fail with no-op).
  - `sort_order_preserved`: build a vec of entries, sort by `raw_score`, assert same order as sorted by `display_score`.
- [ ] Commit: `test(hotspots): red — log-scale normalization and sub-columns`

## Phase 2 — Green (Implementation)

- [ ] Implement `normalize_score(raw: f64) -> f64 { raw.ln_1p() }` in `src/impact/hotspots.rs`.
- [ ] Extend `HotspotEntry` (or equivalent) with `display_score: f64`, `complexity: u64`, `frequency: u64`.
- [ ] Populate `display_score = normalize_score(raw_score)` when constructing entries.
- [ ] Populate `complexity` and `frequency` from the existing scoring computation (they should already be in scope; extract them into the struct).
- [ ] Update human output table: add `Complexity` and `Frequency` columns, use `display_score` in the `Score` column.
- [ ] Update JSON output (if present): add `raw_score` and `display_score` fields.
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `feat(hotspots): log1p score normalization with complexity/frequency sub-columns (CG-9)`

## Verification

- [ ] `changeguard hotspots` — confirm the score column no longer shows a 22× gap between rank 1 and rank 2.
- [ ] `changeguard hotspots` — confirm `Complexity` and `Frequency` columns are present.
- [ ] Verify `src/commands/verify.rs` still ranks #1 (rank order preserved).

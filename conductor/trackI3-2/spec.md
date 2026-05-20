# Track I3-2: Hotspot Score Log-Scaling

**Milestone:** I — Issue Remediation  
**Phase:** 3 — Feature Depth  
**Issue:** CG-9  
**Status:** In Planning

## Objective

The hotspot score column is visually misleading: `src/commands/verify.rs` scores 0.135 while the #2 file scores 0.006 — a 22× gap. The raw score is `complexity × frequency / normalization_factor`. Apply `log1p` normalization to compress extreme outliers into a readable scale while preserving relative ranking. Add raw factor sub-columns so the data is not hidden.

## Requirements

### Score Transformation
In `src/impact/hotspots.rs`, apply the following transformation to the displayed score:
```rust
fn normalize_score(raw_score: f64) -> f64 {
    raw_score.ln_1p()  // ln(1 + x) — compresses large values, 0 maps to 0
}
```

The `HotspotEntry` struct (or equivalent) should carry both:
- `raw_score: f64` — the original value (unchanged, used internally)
- `display_score: f64` — the log-scaled value (used for output only)

### Table Output: Sub-Columns
Extend the hotspot table to include `Complexity` and `Frequency` as separate columns:

```
File                              Score    Complexity  Frequency  Reason
src/commands/verify.rs            0.497    224         21         High complexity + frequent changes
src/commands/watch.rs             0.033    27          9          ...
```

The `Score` column shows `display_score`. `Complexity` and `Frequency` are the raw factors.

### JSON Output
When `--json` is used (if `hotspots` supports it), emit both `raw_score` and `display_score` fields. Do not break existing JSON consumers — add fields, do not remove.

### Sorting
Sorting remains on `raw_score` (not `display_score`) to preserve deterministic rank order. The log transform is monotonic so the rank is identical, but using `raw_score` for sort avoids floating-point surprises with very small deltas.

## API Contract

`HotspotEntry` struct changes:
```rust
pub struct HotspotEntry {
    pub file: String,
    pub raw_score: f64,        // existing field (may be renamed from `score`)
    pub display_score: f64,    // new: log-scaled for output
    pub complexity: u64,       // new: raw complexity factor
    pub frequency: u64,        // new: raw frequency factor
    pub reason: String,        // existing
}
```

## Testing Strategy

- Unit test `log_scale_zero`: `normalize_score(0.0) == 0.0`.
- Unit test `log_scale_positive`: `normalize_score(x) > 0.0` for `x > 0.0`.
- Unit test `log_scale_compresses_outlier`: two scores `0.135` and `0.006`; after normalization the ratio should be < 5× (was 22×).
- Unit test `sort_order_preserved`: a list of `HotspotEntry` sorted by `raw_score` descending equals the same list sorted by `display_score` descending.

## Out of Scope

- Percentile-based ranking (log1p is sufficient and simpler).
- Historical comparison of scores across runs.
- No change to how `raw_score` is computed.

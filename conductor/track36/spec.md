# Track 36: Critical Remediation & Green CI

## Objective
Restore build hygiene by achieving green CI gates (`cargo fmt` and `cargo clippy --all-features`), fix silent error drops in hotspot scoring, and ensure predictive verification degradation is visible to the user, not just logged. 

## Requirements
1. **Green CI Gates**:
   - `cargo fmt --check` MUST pass without formatting diffs across the entire workspace.
   - `cargo clippy --all-targets --all-features -- -D warnings` MUST pass without collapsible `if` warnings, `map` suggestions, or any other warnings.
2. **Hotspot Error Handling**:
   - `src/impact/hotspots.rs` must not use `.filter_map(|res| res.ok())` to silently drop SQLite row errors.
   - Any malformed SQLite rows encountered during hotspot calculation should be propagated or explicitly reported as partial data degradation, following the determinism and error visibility principles in `docs/Engineering.md`.
3. **Prediction Visibility**:
   - `PredictionResult` warnings currently sent only to `tracing::warn!` must be serialized into the `latest-verify.json` verification report.
   - A `prediction_diagnostics` or `warnings` array MUST be exposed in the report schema so that users understand why a prediction might be degraded or incomplete.
4. **No Production Unwraps**:
   - Abide by the idiomatic Rust standard defined in `docs/Engineering.md`. No new unwraps may be introduced in the remediation logic. Errors should be wrapped with context using `anyhow` or `miette`.

## Context
Audit 4 identified these as the most critical blocking issues preventing Phase 2 from reaching "engineering-complete". This track establishes a stable baseline for the rest of Milestone J (Tracks 37-40).
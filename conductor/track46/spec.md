# Track 46: Preserve Enrichment Risk Signals in analyze_risk()

## Overview
The `analyze_risk()` function in `src/impact/analysis.rs` unconditionally recomputes `packet.risk_level` from scratch (line 650) at the end of its analysis, discarding any prior risk elevation applied by enrichment providers. The `ServiceProvider` enrichment in `src/impact/enrichment/services.rs` populates `service_map_delta` with `affected_services` and `cross_service_edges`, and `analyze_risk()` does add service-map risk reasons (lines 476-494) — but if the enrichment pipeline raised `risk_level` before `analyze_risk` runs, that elevation is silently overwritten. The same applies to `risk_reasons`: enrichment-added reasons are preserved via `extend()`, but only if the local `reasons` vec is non-empty; if `reasons` is empty and `packet.risk_reasons` is also empty, a generic "Minimal changes detected" is pushed, overwriting any enrichment context.

## Objectives
- Ensure `analyze_risk()` preserves enrichment-set `risk_level` when it was elevated above the rule-based computation.
- Guarantee that enrichment-added `risk_reasons` are never dropped when the rule-based path produces an empty reasons list.
- Add an end-to-end test proving service-map reasons appear in the final packet after `analyze_risk()`.

## Success Criteria
- `packet.risk_level` after `analyze_risk()` is `max(enrichment_level, rule_based_level)`.
- Enrichment-added `risk_reasons` survive `analyze_risk()` even when the rule-based `reasons` vec is empty.
- New test: index → impact pipeline confirms service-map risk reasons in final output.
- Existing tests pass with zero changes to scoring thresholds.
- CI gate passes.

## Architecture
- `src/impact/analysis.rs` — `analyze_risk()` function (lines 46-667). Only the risk-level assignment (~line 650) and empty-reasons handling (~line 658) change.
- No new modules, no API changes.

## Testing Strategy
- **Red commit**: Write a test that pre-populates `packet.risk_reasons` and `packet.risk_level` (simulating enrichment), runs `analyze_risk()`, and asserts they are preserved.
- **Green commit**: Fix `analyze_risk()` to preserve elevation. Verify all tests pass.

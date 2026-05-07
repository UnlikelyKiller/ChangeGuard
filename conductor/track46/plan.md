# Implementation Plan - Track 46: Preserve Enrichment Risk Signals

## Goal
Fix `analyze_risk()` so enrichment-elevated risk levels and reasons are never silently overwritten.

## Proposed Changes

### 1. Risk Level Preservation [src/impact/analysis.rs]
- At line 650, before overwriting `packet.risk_level`, save the pre-existing level:
  ```rust
  let rule_level = if total_weight > 50 { RiskLevel::High } ...
  packet.risk_level = std::cmp::max(packet.risk_level.clone(), rule_level);
  ```
  where `RiskLevel` derives `Ord` with `Low < Medium < High`.
- This ensures if an enrichment provider already set `High`, it stays `High`.

### 2. Risk Reasons Preservation [src/impact/analysis.rs]
- At line 658, change the empty-reasons logic:
  ```rust
  if reasons.is_empty() && packet.risk_reasons.is_empty() {
      packet.risk_reasons.push("Minimal changes detected".to_string());
  } else {
      packet.risk_reasons.extend(reasons);
  }
  ```
  This is already correct for the `else` branch — enrichment reasons in `packet.risk_reasons` survive when `reasons` is non-empty. The only gap is when `reasons` is empty but `packet.risk_reasons` already has enrichment entries: they are preserved. Code is actually fine here. Focus on the risk_level fix.

### 3. End-to-End Test [src/impact/analysis.rs tests]
- `test_enrichment_risk_level_preserved`:
  - Pre-set `packet.risk_level = RiskLevel::High` and `packet.risk_reasons = vec!["multi-service blast radius".into()]`.
  - Call `analyze_risk()` with a low-weight config (producing Medium or Low).
  - Assert `packet.risk_level == RiskLevel::High` and enrichment reasons intact.

### 4. Verify No Regression
- Ensure all existing `analyze_risk` tests pass unchanged.
- The score thresholds (50/20) must remain identical.

## Verification Plan

### Automated Tests
- `cargo test impact::analysis::tests`
- `cargo test --workspace`

## Definition of Done (DoD)
- [x] **Risk Level Preservation**: `analyze_risk()` never downgrades a High set by enrichment.
- [x] **End-to-End Test**: Test proves service-map elevation survives analysis.
- [x] **Zero Regression**: All existing tests pass with identical scoring.
- [x] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

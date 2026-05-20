# Specification: Predictive Verification IPC & Watcher Intervention (Track C3)

## Overview
Expose ChangeGuard's predictive verification engine to AI-Brains via IPC and extend the file watcher to push high-risk temporal coupling alerts that trigger AI-Brains' RiskReviewAgent.

## Architecture & SRP
- **Module**: `src/verify/mod.rs` and `src/bridge/notify.rs`
- **Responsibility**: Provide a callable predictive failure endpoint for AI-Brains' capture gate; extend watcher to emit risk alerts on dangerous coupling patterns.

## Requirements

### Predictive Verification Endpoint
- Expose a `changeguard verify` mode callable via IPC from AI-Brains that returns a structured `BridgeRecord::VerifyOutcome` with failure probability, drift status, and risk level.
- Must accept a scope parameter (files/directories) to constrain verification to the active change set.
- Must be fast enough to serve as an inline gate — target < 500ms for typical verification plans.
- Results must be deterministic for the same input state.

### Watcher Risk Alerts
- Extend `src/bridge/notify.rs` with a `push_risk_alert()` function that emits `BridgeRecord::RiskAlert` when the watcher detects temporal coupling above a configurable threshold (default: 90%).
- Risk alerts must include: the coupled file paths, coupling score, affected symbols, and a suggested remediation scope.
- Alerts must be fire-and-forget — IPC failures must not crash the watcher.
- The watcher must not alert on the same coupling pair more than once per session (deduplication).

### Design Decision
- MADR formatting is performed by AI-Brains. ChangeGuard sends only structured fields.
- AI-Brains implements its own Datalog translation layer. ChangeGuard exposes the CozoDB run_script interface (already available) rather than pre-defined query endpoints.

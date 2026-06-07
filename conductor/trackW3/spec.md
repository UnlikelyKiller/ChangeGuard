# Track W3 Spec: ADR Lifecycle and Decision Governance

## Background

ADR and decision tracking currently scores 6/10. ChangeGuard can generate MADR from ledger transactions and detect staleness, but ADR lifecycle metadata is weak and decisions are not deeply linked to governed entities.

## Objective

Raise ADR tracking to 9/10 by adding structured lifecycle metadata, decision links, status transitions, stale review handling, and impact warnings when code touches governed entities.

## Proposed Design

1. Add structured ADR fields: status, owner, reviewers, supersedes, superseded_by, affected_entities, decision_scope, reviewed_at, and review_interval_days.
2. Store ADRs as graph nodes linked to ledger transactions, services, endpoints, modules, config keys, data models, tests, and security boundaries.
3. Add `ledger adr update-status`, `ledger adr link`, and `ledger adr review`.
4. Add stale, expired, contradictory, and superseded-decision checks to impact analysis.
5. Preserve existing MADR generation while allowing structured metadata round-trip.

## Critical Files

| File | Expected work |
|---|---|
| `src/ledger/adr.rs` | Extend ADR model and rendering |
| `src/commands/ledger_adr.rs` | Add lifecycle commands |
| `src/ledger/db.rs` | Persist structured ADR metadata |
| `src/impact/enrichment/` | Add decision governance checks |
| `docs/` | Document ADR status and review workflow |

## Definition of Done

- ADR metadata can be created, updated, searched, exported, and linked without editing ledger state files directly.
- Superseded and stale ADRs are clearly represented in human and JSON output.
- Changes touching governed entities surface active, stale, or conflicting decisions.
- Existing ledger-backed ADR generation remains backward-compatible.
- Target score after completion: 9/10.

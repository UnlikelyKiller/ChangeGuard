# Track W11 Spec: Hotspot and Temporal Coupling Trends

## Background

Hotspot and temporal coupling tracking currently scores 9/10. ChangeGuard already has strong hotspot and co-change analysis. The target is 10/10 through operational polish, trend history, budgets, and ownership links.

## Objective

Raise hotspot and temporal coupling tracking to 10/10 by persisting snapshots over time, linking hotspots to owners/services/tests, adding budgets, and improving explainability.

## Proposed Design

1. Persist hotspot and temporal coupling snapshots with timestamp, commit, branch, churn, complexity, score, owner, and service links.
2. Add trend delta detection to show improving, worsening, new, and resolved hotspots.
3. Add owner/service/test links to hotspots using W1 graph relations.
4. Add hotspot budgets and policy thresholds per directory or service.
5. Add `hotspots trend`, `hotspots explain`, and `hotspots budget`.

## Critical Files

| File | Expected work |
|---|---|
| `src/impact/hotspots.rs` | Persist and explain hotspot scores |
| `src/impact/temporal.rs` | Persist temporal coupling snapshots |
| `src/commands/hotspots.rs` | Add trend/explain/budget output |
| `src/config/model.rs` | Add hotspot budget policy config |
| `src/verify/predict.rs` | Use trend and budget data for verification hints |

## Definition of Done

- Hotspot and coupling history can be queried across commits and branches.
- Budget violations are deterministic and configurable.
- Trend output shows new, worsening, improving, and resolved risk.
- Target score after completion: 10/10.

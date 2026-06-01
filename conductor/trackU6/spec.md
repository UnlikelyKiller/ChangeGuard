# Track U6 Spec: Path-Weighted Risk Scoring

## Background
ChangeGuard currently treats all file changes with equal weight when calculating the `OVERALL RISK`. A change to a `.md` file can contribute as much to the "churn" and "risk" signals as a change to a `.rs` file, which is often incorrect for software engineering risk.

## Objective
Implement a weighting system that assigns different risk multipliers based on file extensions or path patterns.

## Proposed Design
* Add `risk_weights` to `[impact]` configuration in `config.toml`.
* Proposed tiered weighting model (from internal audit):
  * `.rs`: 1.0 (Logic)
  * `.toml`: 0.8 (Core Config)
  * `.json`: 0.7 (Data Schemas)
  * `.yml`, `.yaml`: 0.3 (Service Config)
  * `.md`, `.txt`: 0.1 (Docs)
  * `.codex`, `.claude`: 0.01 (External Data)
* Update `ImpactOrchestrator` to apply these weights to the `score` contribution of each changed file.
* Reflect these weights in the `RiskReason` output (e.g., "High-weight logic file modified").

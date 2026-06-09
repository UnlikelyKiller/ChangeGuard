# Track K6: Temporal Risk Precision (Time-Bounded Hotspots)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`changeguard hotspots` analyzes the entire git history by default (or a large fixed chunk). This often surfaces "historical" risk areas that are no longer relevant to current development sprints, diluting the signal for where brittle code is *currently* located.

## Solution
1. **Time-Bounded Analysis**:
    - Add a `--commits <N>` flag to the `hotspots` command (e.g., `--commits 100`).
    - Add a `--days <N>` flag to analyze only the last N days of change frequency.
2. **Dynamic Weighting**:
    - Weight recent commits more heavily than older ones in the hotspot score calculation.
3. **Comparison Mode**:
    - Add a `--since <REF>` flag to compare current hotspots against a baseline (e.g., `main` branch).

## Definition of Done (DoD)
- [ ] `changeguard hotspots --commits 50` calculates risk density based only on the last 50 commits.
- [ ] `changeguard hotspots --days 7` focuses on the last week of activity.
- [ ] Output includes a clear header: `[Analysis Range: Last 50 commits]`.
- [ ] CI gate passes.

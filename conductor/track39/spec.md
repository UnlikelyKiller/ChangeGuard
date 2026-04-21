# Track 39: Dependency & Federation Deepening

## Objective
Close the remaining Track 32 and Track 33 audit gaps from `docs/audit4.md`: use current repository dependency data for prediction, make verification degradation visible, broaden federation dependency discovery, automate sibling discovery during impact, and replace ad hoc federation export redaction.

## Requirements
- Build predictive structural edges from current repository files when `verify` runs.
- Recompute temporal coupling for `verify` when the latest packet lacks temporal data.
- Persist all prediction degradation warnings into `latest-verify.json`.
- Avoid silent storage/history degradation in `verify`.
- Make `impact` refresh federation links/dependencies from sibling schemas before cross-repo impact checks when possible.
- Discover federated dependencies from the current repository, not only latest changed files.
- Use the shared redaction model for federated export filtering.
- Add federation tests for schema version rejection, sibling cap, and symlink/path confinement behavior.

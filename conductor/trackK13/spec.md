# Track K13: Index Freshness Recovery Workflow

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`index --check` correctly reports stale index state, but the remediation path is not strong enough for users and downstream commands. Strict index consumers fail until users know to run `index --incremental`.

## Objective
Make stale-index output more actionable and make freshness recovery consistent across commands that depend on index state.

## Scope
- Show a concise list of representative stale files.
- Surface the exact recommended command in human and JSON output.
- Audit `--auto-index` availability across search, ask, dead-code, hotspots, and semantic paths.
- Clarify strict versus advisory freshness semantics.

## Non-Goals
- Do not make `index --check` mutate state.
- Do not force auto-indexing for commands where the user did not request it.
- Do not rebuild heavyweight semantic or SCIP indexes implicitly unless the command explicitly opted into that behavior.

## Implementation Notes
- JSON output should remain stable and include fields that scripts can consume: stale count, sample paths, strict status, and recommended command.
- Human output should explain that stale is advisory unless `--strict` is used.
- Auto-index behavior should distinguish lightweight incremental source indexing from expensive semantic indexing.

## Success Criteria
- [ ] `index --check` human output shows count, sample stale paths, and the exact recovery command.
- [ ] `index --check --json` includes stale sample paths and a recommended action field.
- [ ] Index-dependent commands either support `--auto-index` or clearly explain why they do not.
- [ ] Tests cover stale, missing, and current index states.
- [ ] CI gate passes.

## Definition of Done
- [ ] `index --check` exits 0 for advisory stale state and `index --check --strict` exits 1 for the same state.
- [ ] `index --check --json` validates as structured JSON with stale samples and recommended action.
- [ ] `search`, `ask`, `dead-code`, and `hotspots` have consistent help/output around stale index recovery.
- [ ] Tests cover stale, missing, corrupt, and current index states.
- [ ] `changeguard verify` passes.
- [ ] `cargo install --path . --force` succeeds and installed-binary index smoke checks pass.

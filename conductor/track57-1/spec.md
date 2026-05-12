# Track 57-1: Dependency Alert Remediation

## Objective

Resolve the ChangeGuard-owned Dependabot alert for `lru` and consume the
CozoDB-redux-owned `lz4_flex` transitive fix once available. Preserve
ChangeGuard's local-first behavior and current search functionality.

## Alerts

- `lru` low severity: vulnerable `0.12.5` is pulled by `tantivy 0.22.1`.
- `lz4_flex` high severity: vulnerable `0.10.0` was pulled by
  `swapvec -> cozo`. CozoDB-redux fixed this at `6690fdac` by making the
  vendored `swapvec` path transitive to downstream git consumers.

## Requirements

- Research current upstream versions and compatibility before changing dependencies.
- Upgrade ChangeGuard's direct `tantivy` dependency to a release that pulls a
  fixed `lru` version.
- Avoid lockfile-only churn that Cargo cannot legally resolve.
- Keep `.agents/skills/changeguard/SKILL.md` aligned with the dependency-alert workflow.
- Capture CI failure context and account for it in the implementation plan.
- Do not add a downstream `swapvec` patch in ChangeGuard; consume the
  CozoDB-redux git fix through `Cargo.lock`.
- Verify with `cargo fmt`, `cargo clippy`, workspace tests, and `changeguard verify`.

## Non-Goals

- Do not replace CozoDB storage.
- Do not redesign the search subsystem.
- Do not suppress Dependabot alerts without dependency evidence.
- Do not implement or fork the CozoDB-redux `swapvec/lz4_flex` fix in this
  repository.

## Testing Strategy

- `cargo check` after the `tantivy` bump.
- Targeted search tests for Tantivy API/index behavior.
- Full `changeguard verify` before completion.

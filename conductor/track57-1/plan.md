# Track 57-1 Plan

- [x] Capture dependency matrix for `cozo`, `swapvec`, `lz4_flex`, `tantivy`, and `lru`.
- [x] Inspect latest CI failure and determine whether it blocks dependency remediation.
- [x] Select the smallest compatible ChangeGuard-owned dependency upgrade path.
- [x] Update `Cargo.toml`/`Cargo.lock` for `tantivy` and code as needed.
- [x] Record CozoDB-redux `swapvec/lz4_flex` as an external dependency handoff.
- [x] Consume CozoDB-redux `6690fdac` after the transitive `swapvec/lz4_flex` fix landed.
- [x] Update `.agents/skills/changeguard/SKILL.md` with dependency-alert guidance.
- [x] Stabilize the failing Linux CI watcher test discovered during dependency work.
- [x] Run targeted dependency/search checks.
- [x] Run `changeguard verify`.
- [x] Mark the track complete and report residual risk.

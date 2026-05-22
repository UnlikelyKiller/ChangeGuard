# Track K10 Plan: Ignore-Aware Scan Cleanliness

## Phase 1: Reproduce and Characterize
- [ ] Add a scan fixture with ignored `.codex/` and non-ignored untracked files.
- [ ] Assert ignored tool directories do not mark the scan dirty.
- [ ] Assert non-ignored untracked files still appear as changes.

## Phase 2: Filtering Semantics
- [ ] Centralize ignore matching for scan and impact inputs.
- [ ] Apply repo config ignore patterns before unsupported-language analysis.
- [ ] Preserve tracked changes even when their path matches an ignore pattern.
- [ ] Avoid reporting newly added directories as extensionless files when their child files can be reported.
- [ ] Add an opt-in flag or diagnostic mode for showing ignored artifacts.

## Phase 3: Verification
- [ ] Run `changeguard scan` in this repo with ignored agent directories present.
- [ ] Run `changeguard scan --impact` and verify no `.claude`/`.codex` temporal-coupling noise.
- [ ] Confirm conductor markdown additions are reported at file granularity.
- [ ] Run `cargo install --path . --force` and repeat installed-binary scan checks.
- [ ] Run full CI gate.

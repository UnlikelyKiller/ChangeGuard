# Track K10: Ignore-Aware Scan Cleanliness

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`changeguard scan` reports the repository as dirty when Git itself is clean because ignored local tool directories such as `.claude/` and `.codex/` are surfaced as added unsupported files. This makes the core safety signal noisy for agent-driven workflows.

## Objective
Make scan and impact output align with actionable repository state by honoring ignored local tooling paths by default and separating ignored untracked artifacts from real drift.

## Scope
- Apply configured ignore patterns and Git ignore status before rendering scan changes.
- Preserve an explicit diagnostic path for users who want to inspect ignored local artifacts.
- Ensure impact analysis does not report unsupported-language warnings for ignored agent directories.

## Non-Goals
- Do not hide tracked file modifications, even if the path also matches an ignore pattern.
- Do not change ledger drift semantics for tracked files.
- Do not hard-code only `.claude` and `.codex`; the implementation should respect configured ignore patterns.

## Implementation Notes
- Prefer Git's ignored/untracked classification when available, then layer ChangeGuard config ignores on top.
- Directory-level changes should be expanded or classified consistently enough that `conductor/trackK10` does not appear as an unsupported extensionless file when only markdown files were added underneath it.
- Add a regression for the current repo shape: ignored agent directories exist while conductor markdown changes are real changes.

## Success Criteria
- [ ] A clean Git worktree with ignored `.claude/`, `.codex/`, or `.opencode/` content reports `State: CLEAN`.
- [ ] `scan --impact` excludes ignored agent directories from changed-file and temporal-coupling output.
- [ ] A test fixture covers ignored untracked files and non-ignored untracked files side by side.
- [ ] Help or output text documents how to include ignored artifacts when needed.
- [ ] CI gate passes.

## Definition of Done
- [ ] `changeguard scan` matches `git status --short` for actionable changes in this repository while `.claude/` and `.codex/` remain present.
- [ ] `changeguard scan --impact` reports conductor markdown files as docs/planning changes, not unsupported extensionless directories.
- [ ] New tests prove ignored untracked paths are filtered and non-ignored untracked paths are preserved.
- [ ] `changeguard verify` passes.
- [ ] `cargo install --path . --force` succeeds and the installed binary passes the same scan smoke checks.

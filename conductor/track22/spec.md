# Specification: Track 22 — Structural Completion and Plan Reconciliation

## Overview
Address the remaining non-trivial gaps from `docs/audit2.md` that are not purely command-behavior bugs: scan diff-summary integration, incomplete storage seams, missing planned modules, missing plan-era docs artifacts, and remaining test/release readiness gaps.

This track must reconcile the plan with the actual product without forcing speculative architecture solely to satisfy an old file list.

## Breaking-Risk Assessment
This track carries the highest risk of accidental overengineering and accidental breakage.

Required guardrails:

- prefer additive seams over moves/renames
- do not introduce DB schema or report changes without a concrete consumer
- do not change default scan verbosity in a way that makes normal use noisy
- when a plan-listed file is still unnecessary, document the deviation instead of inventing a fake subsystem

## 1. Scan Diff-Summary Integration
**Priority: MEDIUM**

### Current State
- `src/git/diff.rs` exists
- `scan` does not expose diff summaries to users or reports

### Required Outcome
- integrate diff-summary support into scan output or scan-adjacent reporting in a bounded way
- keep output concise and deterministic
- do not dump full diffs by default; use a capped summary
- prefer report enrichment or an opt-in summary section over noisy default console output

## 2. Index Storage and Symbol Persistence Decision
**Priority: MEDIUM**

### Current Gap
- `src/index/storage.rs` is missing
- SQLite schema lacks a dedicated `symbols` table despite the plan calling for one

### Required Outcome
This track must make an explicit decision, backed by current usage:

- create `src/index/storage.rs` as the storage seam if symbol persistence has a real consumer now, and
- add a minimal `symbols` table only if the current product genuinely queries or benefits from persisted symbol data

If there is still no concrete consumer:

- add the seam without a speculative heavy implementation, or
- document the deliberate deferral in the relevant architecture/conductor docs

Keep scope narrow: changed-file symbols only, not whole-repo indexing.

## 3. Remaining Planned Module Gaps
**Priority: MEDIUM**

### Missing or Collapsed Files
- `src/index/normalize.rs`
- `src/gemini/wrapper.rs`
- `src/output/table.rs`
- `src/util/fs.rs`
- `src/util/hashing.rs`
- `src/util/process.rs`
- `src/util/text.rs`
- `src/state/locks.rs`

### Required Direction
For each remaining gap, do one of:

- implement the minimal file/module needed because there is already duplicated responsibility, or
- add a thin shim/re-export so the planned architecture is represented explicitly, or
- document a deliberate YAGNI deferral in the relevant spec/architecture docs

This track must not add abstraction for its own sake. Thin seams are acceptable. Fake subsystems are not.

## 4. Documentation Reconciliation
**Priority: MEDIUM**

### Required Docs
- create `docs/prd.md`
- add `docs/implementation-plan.md` or a clearly documented alias/redirect to `docs/Plan.md`

### Goal
Make the repo layout match what the plan says future contributors should expect, without moving the current canonical plan unexpectedly.

## 5. Test and Release Readiness Gaps
**Priority: MEDIUM**

### Required Additions
- black-box CLI tests that invoke the compiled binary for at least one or two core flows
- stronger scan-output assertions rather than only `is_ok()`
- add `cargo deny` to CI or document why it is intentionally deferred

### Hardening Requirement
- any new scan/report assertions must avoid volatile timestamps or nondeterministic ordering

## 6. State Locking Decision
**Priority: LOW**

### `src/state/locks.rs`
- The plan names it, but current code does not materially need a lock manager yet
- This track may implement a minimal placeholder/seam or explicitly document deferral
- Do not build a sophisticated lock manager without a demonstrated race

## Non-Goals
- repo-wide semantic indexing
- a generalized plugin or extension architecture
- DB-first persistence for data that has no concrete query path yet
- reshaping the codebase just to mirror the old file list

## Verification
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test -j 1 -- --test-threads=1`

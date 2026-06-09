# Specification: Track 19 — Reset and Recovery Completion

## Overview
Address the largest remaining functional gap from `docs/audit2.md`: `reset` is declared in the CLI and documentation but is not implemented. This track completes the repo-local recovery story required by the plan without widening the command into a general-purpose cleanup tool.

## Breaking-Risk Assessment
This track is intended to be non-breaking at the CLI surface:

- `changeguard reset` remains the canonical command name
- new flags must be additive only
- the default behavior must preserve user-authored config and rules
- report/config paths must not move

The only intentional behavior change is that `reset` stops being a stub and begins performing the documented recovery action.

## 1. Real Reset Command
**Priority: HIGH** — `docs/Plan.md` requires a real reset command and recovery path.

### Required Files
- Create `src/commands/reset.rs`
- Register `pub mod reset;` in `src/commands/mod.rs`
- Move reset command execution out of `src/cli.rs` so the CLI only routes to `commands::reset::execute_reset(...)`

### CLI Shape
Reset should default to deleting derived state only and preserving user-authored repo-local configuration.

Required flags:

- `changeguard reset`
  - removes derived state under `.changeguard/logs/`, `.changeguard/tmp/`, `.changeguard/reports/`, `.changeguard/state/`
  - removes SQLite artifacts such as `ledger.db`, `*.wal`, `*.shm`
  - preserves `.changeguard/config.toml` and `.changeguard/rules.toml`
- `changeguard reset --remove-config`
  - also removes `config.toml`
- `changeguard reset --remove-rules`
  - also removes `rules.toml`
- `changeguard reset --all`
  - removes the entire `.changeguard/` tree

Hardening requirement:

- destructive deletion of user-authored files must require explicit intent
- `--all`, `--remove-config`, and `--remove-rules` must be gated behind a confirmation flag such as `--yes` or an equivalent non-interactive confirmation mechanism
- plain `changeguard reset` must remain non-interactive so existing automation is not blocked

## 2. Recovery Semantics
**Priority: HIGH**

### Explicit Recovery Story
- Reset must be the documented and implemented recovery path for:
  - bad cached reports
  - corrupted or stale SQLite state
  - stale watch batches
  - stale WAL/SHM artifacts

### Scope
- This track does not need automatic DB corruption detection in all commands
- It must make recovery possible and explicit once the user decides to reset
- It must not silently recreate config/rules during reset itself; recreation belongs to `init`

## 3. Safety and Boundedness
**Priority: HIGH**

### Required Safeguards
- All filesystem mutations must stay strictly under the resolved `.changeguard/` root
- The implementation must validate target paths before recursive deletion
- Unknown siblings outside `.changeguard/` must never be touched
- Missing files and directories are normal and must not be treated as errors

### Deletion Strategy
- Enumerate removal targets deterministically
- Prefer deleting known derived subtrees individually for default reset
- Reserve whole-tree removal for `--all`
- If any deletion fails, continue best-effort for the remaining in-scope targets, then return a non-zero result with a deterministic summary of failures

### Platform Requirements
- Windows-safe handling for readonly files, WAL/SHM artifacts, and paths with spaces
- Close DB/storage handles before attempting deletion
- Avoid shelling out for file removal

## 4. State Layout Alignment
**Priority: MEDIUM**

### Required Cleanup Targets
- `.changeguard/reports/latest-impact.json`
- `.changeguard/reports/latest-verify.json`
- `.changeguard/state/current-batch.json`
- `.changeguard/state/ledger.db`
- `.changeguard/state/ledger.db-wal`
- `.changeguard/state/ledger.db-shm`
- any other derived files under logs/tmp/reports/state

### Preserve-by-Default Targets
- `.changeguard/config.toml`
- `.changeguard/rules.toml`

## 5. Output and Error Quality
**Priority: MEDIUM**

- Use actionable errors via `miette`
- Name the path that failed to delete when possible
- Distinguish “already absent” from “removed” from “failed to remove”
- Summarize preserved, removed, and failed artifacts deterministically
- The final summary must be stable enough for tests

## 6. Tests
**Priority: HIGH**

### Required Coverage
- reset with no `.changeguard/` present
- reset after `init` + generated state files
- reset preserves config/rules by default
- `--remove-config`
- `--remove-rules`
- `--all`
- repeated reset remains safe
- reset clears SQLite db, wal, shm files
- destructive modes reject execution without explicit confirmation
- path-boundedness test that proves nothing outside `.changeguard/` is touched

### Nice-to-Have
- integration test that creates reports and watch batch JSON, then verifies they are removed
- readonly-file test on Windows-friendly semantics

## Non-Goals
- automatic repair of a live open database connection
- global repo cleanup outside `.changeguard/`
- interactive wizard UX

## Verification
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test -j 1 -- --test-threads=1`

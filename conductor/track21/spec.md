# Specification: Track 21 — Verification Process Hardening

## Overview
Address the remaining verification and cross-platform hardening gaps from `docs/audit2.md`. The current verify flow works, but it still relies on shell-string composition and does not use the existing process-policy seam.

This track should harden execution semantics without breaking existing rules files or report consumers.

## Breaking-Risk Assessment
This track should preserve existing verification inputs and outputs unless there is a clear safety reason not to:

- existing `required_verifications` string commands in rules must continue to work
- manual `--command` must continue to accept a free-form command string
- report JSON should remain backward compatible where possible
- any future structured representation must be additive, with legacy string commands still supported

## 1. Dedicated Verify Runner Layer
**Priority: HIGH**

### Required Files
- Create `src/verify/runner.rs`
- Create `src/verify/timeouts.rs`
- Register both in `src/verify/mod.rs`

### Responsibilities
- `plan.rs`: deterministic step selection only
- `runner.rs`: command execution, timeout handling, result capture, policy checks
- `timeouts.rs`: timeout defaults and policy helpers
- `commands/verify.rs`: orchestration only

This restores the SRP boundary expected by the plan and the engineering doc.

## 2. Reduce Shell Dependence Without Breaking Rule Compatibility
**Priority: HIGH**

### Current Problem
`commands/verify.rs` shells out via `cmd /C` or `sh -c` for all verification commands.

### Required Direction
- Use direct process invocation wherever the command shape is known and structured
- Keep shell fallback only for explicitly manual free-form commands or legacy commands that cannot be safely structured yet

### Compatibility Model
- Keep `VerificationStep.command: String` for compatibility unless there is a compelling reason to version the schema
- A structured execution model may be introduced internally or additively, for example:
  - executable
  - args
  - timeout
  - description
  - execution_mode
- Rule-driven commands should prefer direct process execution when they can be losslessly tokenized
- If a rule command cannot be safely tokenized cross-platform, execute it via an explicitly labeled shell fallback path and record that fact

### Hardening Requirements
- no shell fallback for commands that the runner can represent safely as `Command + args`
- manual `--command` remains the explicit shell path unless and until the CLI surface is deliberately expanded
- environment handling must be deterministic enough for tests
- prefer explicit `current_dir`, `stdin`, `stdout`, and `stderr` policy in the runner

## 3. Enforce Process Policy
**Priority: HIGH**

### Current Problem
`src/platform/process_policy.rs` exists but is unused.

### Required Outcome
- verification must call `check_policy(...)` before executing a step
- denied commands must fail with actionable diagnostics
- default policy may remain permissive, but the seam must become real
- policy checks must operate on the actual executable identity used for execution, not just an imprecise display string

## 4. Timeout Policy Centralization
**Priority: MEDIUM**

### Requirements
- move timeout defaults out of ad hoc literals and into `verify/timeouts.rs`
- distinguish:
  - default timeout for auto-planned steps
  - manual override timeout from CLI
  - future repo-specific timeout extension seam
- timeout behavior must be stable and testable

## 5. Cross-Platform Verification Tests
**Priority: MEDIUM**

### Required Test Additions
- `tests/platform_windows.rs`
- `tests/platform_wsl.rs` or platform-path seam tests that can run conditionally
- verify execution tests that avoid brittle shell assumptions

### Minimum Coverage
- process policy denial path
- timeout path
- direct process execution path
- manual shell command path
- legacy rule-string command path

## 6. Documentation
**Priority: LOW**

- Update verification-related docs/comments to reflect structured execution vs manual shell fallback
- Make it explicit which path is used for rule-driven steps vs manual `--command`
- Document any still-shell-based legacy path as an intentional compatibility compromise, not an accident

## Non-Goals
- inventing a full shell parser
- breaking existing rule syntax in order to force a structured config format
- tightening the default policy so far that normal `cargo`-based usage breaks

## Verification
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test -j 1 -- --test-threads=1`

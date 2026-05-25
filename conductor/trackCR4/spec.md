# Track CR4: Align Health Check Command Parsing

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
`changeguard verify --health` uses a naive whitespace split (`split_whitespace()`) to determine the executable for each verify step. However, the verification engine runner (`src/verify/runner.rs`) uses more robust parsing and shell wrapper fallbacks. As a result, quoted executables, environment variable prefixes, or shell operators cause false "missing executable" reports in the health check.

## Objective
Unify command parsing between the health checker and the verification command runner to eliminate false negatives.

## Scope
- Refactor the command health check logic in `src/commands/verify.rs` to extract the target executable using the runner's parsing seams.
- Gracefully handle environment variables, shell built-ins, and quoted command paths in the health scanner.

## Success Criteria
- [ ] Complex verification steps (e.g., commands beginning with quotes or environment variables) are correctly parsed by `verify --health`.
- [ ] No false negatives are reported for valid runner commands.
- [ ] Health checks align with actual command execution status.

## Definition of Done
- [ ] Naive whitespace parsing replaced in `src/commands/verify.rs` with robust extraction.
- [ ] Verified against quoted verify steps on Windows.
- [ ] `cargo test` passes.

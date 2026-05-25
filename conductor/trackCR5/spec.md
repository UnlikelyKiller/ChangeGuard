# Track CR5: CLI & Process Hardening Test Coverage

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
1. Newly introduced verify and config CLI flags (`verify --signatures`, `verify --health`, `verify --dry-run`, `config view`) have limited test coverage in the integration test suite.
2. The `update --force-unlock` command kills all processes containing the string `changeguard` machine-wide without checking whether they belong to the current repository, creating unnecessary side-effects.

## Objective
Implement targeted integration tests for the verify and config CLI features, and refine the `update --force-unlock` command to check process owner or lock path context before terminating.

## Scope
- Add integration tests for `--health`, `--dry-run`, and `config view` CLI variations.
- Update `src/commands/update.rs` to refine the force-unlock safety scope, checking path or repository match if feasible, and add corresponding tests.

## Success Criteria
- [ ] Integration tests verify correctness of `verify --health`, `verify --dry-run`, and `config view`.
- [ ] `update --force-unlock` process termination is scoped safely or tested under concurrent scenarios.
- [ ] Crate coverage metrics for verification and config features are improved.

## Definition of Done
- [ ] Integration tests added in `tests/cli_verify.rs` and `tests/cozo_schema_migration.rs` (or similar).
- [ ] `src/commands/update.rs` process killing scoped or contextual safety checks added.
- [ ] All tests compile and pass.

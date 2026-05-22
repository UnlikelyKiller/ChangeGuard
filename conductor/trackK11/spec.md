# Track K11: Read-Only CozoDB Lock Resilience

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
Multiple read-only ChangeGuard commands fail when invoked concurrently because each process attempts to initialize or open the same CozoDB store and exits immediately on the database lock. This is disruptive for agents, scripts, and diagnostic smoke checks.

## Objective
Make read-only command concurrency predictable by waiting briefly, using clearer lock diagnostics, and avoiding exclusive storage initialization where it is not required.

## Scope
- Audit read-only commands that currently initialize CozoDB or full storage.
- Add bounded retry/backoff for transient database-lock acquisition failures.
- Prefer read-only fast paths for commands that do not need graph writes.
- Improve the user-facing error when another ChangeGuard process owns the lock.

## Non-Goals
- Do not introduce unsafe concurrent writes to CozoDB.
- Do not mask real storage corruption as transient lock contention.
- Do not make every command wait indefinitely for a lock.

## Implementation Notes
- Use a small bounded retry budget for read-only commands, with jitter or fixed backoff to avoid synchronized retries.
- Prefer not opening CozoDB at all when a command can answer from Git, config, SQLite, or existing JSON artifacts.
- Keep write paths conservative: commands that mutate graph state should still require exclusive access.

## Success Criteria
- [ ] Concurrent read-only smoke commands no longer fail immediately on CozoDB lock contention.
- [ ] Lock waits are bounded, configurable, and default to a short human-tolerable window.
- [ ] Lock timeout errors name the likely competing process class and suggest rerunning or reducing parallelism.
- [ ] Commands that do not need graph state avoid opening CozoDB.
- [ ] CI gate passes.

## Definition of Done
- [ ] A scripted smoke run starts at least five read-only commands concurrently and produces no immediate CozoDB lock failures.
- [ ] A forced lock-timeout scenario returns a concise actionable error within the configured timeout.
- [ ] Storage initialization tests cover read-only fast paths and write-path exclusivity.
- [ ] `changeguard verify` passes.
- [ ] `cargo install --path . --force` succeeds and installed-binary concurrent smoke checks pass.

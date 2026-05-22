# Track K11 Plan: Read-Only CozoDB Lock Resilience

## Phase 1: Command Inventory
- [ ] List read-only commands that initialize CozoDB, SQLite, or both.
- [ ] Identify commands that can use metadata-only or SQLite-only paths.
- [ ] Add a regression test or smoke harness for two concurrent read-only commands.
- [ ] Define the default lock wait budget and config surface.

## Phase 2: Lock Handling
- [ ] Introduce bounded retry/backoff around CozoDB open.
- [ ] Add command-level classification for read-only storage access.
- [ ] Avoid CozoDB initialization entirely for commands that do not need graph data.
- [ ] Improve lock error wording with actionable recovery guidance.

## Phase 3: Verification
- [ ] Run concurrent read-only smoke checks covering `index --check`, `viz`, `ledger adr`, `federate export`, and `ask --semantic`.
- [ ] Run a forced lock-timeout check and verify the error text.
- [ ] Confirm sequential behavior remains unchanged.
- [ ] Run `cargo install --path . --force` and repeat installed-binary concurrency smoke checks.
- [ ] Run full CI gate.

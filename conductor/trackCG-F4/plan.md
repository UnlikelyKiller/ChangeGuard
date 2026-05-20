## Plan: Fix federate scan Noisy Schema Warnings (Track CG-F4)

### Summary
`changeguard federate scan` logs a `WARN` for every sibling repo that lacks a proper ChangeGuard schema: "Invalid schema at C:\dev\AI-Brains: Invalid schema: ledger entity must not be empty". These are not actionable — the sibling repo is a different tool. The warning should only fire when a repo HAS `.changeguard/` but its schema is malformed.

### Phase 1: Fix
- [ ] Task 1.1: In the federated discovery logic (likely `src/federated/refresh.rs`), check for `.changeguard/` directory before attempting schema validation
- [ ] Task 1.2: Skip repos without `.changeguard/` silently (no warning)
- [ ] Task 1.3: Keep the warning for repos that have `.changeguard/` but whose schema fails validation (actionable)
- [ ] Task 1.4: Or alternatively: downgrade the log level from `WARN` to `DEBUG` for schema validation failures during discovery

### Phase 2: Verify
- [ ] Task 2.1: `changeguard federate scan` with `C:\dev\AI-Brains` as a sibling produces no schema warnings
- [ ] Task 2.2: `changeguard federate scan` still warns if a sibling has `.changeguard/` with invalid schema

### Phase 3: Gate
- [ ] Task 3.1: `cargo fmt --all -- --check` passes
- [ ] Task 3.2: `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Task 3.3: `cargo test --workspace` passes

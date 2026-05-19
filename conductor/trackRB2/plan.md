## Plan: Track R-B2 - Master Remediation
### Phase 1: Fail-Safe Hardening
- [x] Task 1.1: Implement `kill_on_timeout` for `ai-brains recall` in `src/bridge/client/client_cli.rs`.
- [x] Task 1.2: Refactor `src/bridge/ipc.rs` to use a bounded background worker for connect/writes to prevent thread leaks.
- [x] Task 1.3: Update `src/bridge/model.rs` with strict version enforcement.

### Phase 2: Integration Integrity
- [x] Task 2.1: Refactor prompt assembly in `src/commands/ask.rs` to deduplicate and correctly truncate context.
- [x] Task 2.2: Wire `BridgeRecord::Query` (or equivalent) through IPC in `src/bridge/client.rs`.
- [x] Task 2.3: Add a regression test specifically for the "hanging process" case (mocked).

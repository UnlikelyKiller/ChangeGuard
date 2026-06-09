## Plan: Track B4 - Bridge Query Client
### Phase 1: Shell Invocation Fallback
- [x] Task 1.1: Add `query` subcommand to `src/commands/bridge.rs`.
- [x] Task 1.2: Create `src/bridge/client.rs` defining a `query_external_cli` function.
- [x] Task 1.3: Use `std::process::Command` to invoke the external `ai-brains recall` tool.
- [x] Task 1.4: Parse STDOUT stream into `BridgeRecord::Insight` variants.
- [x] Task 1.5: Add a unit test with mocked subprocess execution verifying fail-open logic on binary absence.

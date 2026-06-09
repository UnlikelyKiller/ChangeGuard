# Plan: Track 11 - Ask Gemini Baseline

### Phase 12: Ask Gemini
- [ ] Task 11.1: Create `src/gemini/mod.rs` and `src/gemini/prompt.rs`.
- [ ] Task 11.2: Implement prompt construction logic.
  - [ ] System prompt with ChangeGuard persona.
  - [ ] User prompt with `ImpactPacket` JSON injection.
- [ ] Task 11.3: Implement `run_query` using `ExecutionBoundary`.
  - [ ] Handle `gemini` command execution.
  - [ ] Use a high timeout (e.g. 120s) for LLM responses.
- [ ] Task 11.4: Implement `src/commands/ask.rs`.
  - [ ] Load latest packet from `StorageManager`.
  - [ ] Build full prompt.
  - [ ] Invoke runner.
- [ ] Task 11.5: Register `ask` subcommand in `src/cli.rs`.
- [ ] Task 11.6: Add unit tests for prompt building.
- [ ] Task 11.7: Add integration tests in `tests/cli_ask.rs`.
- [ ] Task 11.8: Final verification with `cargo test -j 1 -- --test-threads=1`.

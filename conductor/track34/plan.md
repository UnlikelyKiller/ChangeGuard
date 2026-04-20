## Plan: Narrative Reporting Completion

### Phase 1: CLI and Core Prompt Wiring
- [ ] Task 1.1: Update `src/cli.rs` to add a `--narrative` boolean flag to the `Ask` command.
- [ ] Task 1.2: Update `src/commands/ask.rs` to map the `--narrative` CLI flag to `GeminiMode::Narrative`.
- [ ] Task 1.3: Update `src/commands/ask.rs` to unconditionally use `NarrativeEngine::generate_risk_prompt(&latest_packet)` when the mode is `GeminiMode::Narrative`, removing the hardcoded `"summary"` query check.

### Phase 2: Token Budgeting and Annotations
- [ ] Task 2.1: In `src/commands/ask.rs`, determine the token budget (e.g., limit of 409,600 characters representing ~102k tokens).
- [ ] Task 2.2: Call `latest_packet.truncate_for_context(409600)` before generating the final user prompt.
- [ ] Task 2.3: If `truncate_for_context` returns `true`, append the exact string `"\n\n[Packet truncated for Gemini submission]"` to the generated prompt.

### Phase 3: Gemini Execution Robustness
- [ ] Task 3.1: Update `src/gemini/wrapper.rs` to spawn the Gemini CLI with the `analyze` argument (`Command::new("gemini").arg("analyze")`).
- [ ] Task 3.2: In `src/gemini/wrapper.rs`, check for `ErrorKind::NotFound` when spawning the process. If encountered, return the exact error message: `"Gemini CLI not found. Install Gemini CLI to enable narrative summaries."`
- [ ] Task 3.3: In `src/commands/ask.rs`, handle failures from `run_query()`. If it returns an error, serialize the `latest_packet` and write it to `.changeguard/reports/fallback-impact.json` (or another appropriate artifact path) as a fallback mechanism, then propagate the original error.

### Phase 4: Golden Prompt Tests
- [ ] Task 4.1: Create a test module or file for golden prompt tests (e.g., in `src/gemini/narrative.rs` or `tests/narrative_golden.rs`).
- [ ] Task 4.2: Build a fully populated, deterministic `ImpactPacket` fixture containing hotspots, temporal couplings, and file changes.
- [ ] Task 4.3: Assert that the output of `NarrativeEngine::generate_risk_prompt` precisely matches a hardcoded or snapshot golden string, ensuring prompt construction stability.
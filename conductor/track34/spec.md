# Track 34: Narrative Reporting Completion

## 1. Goal
Complete the Phase 2 Narrative Reporting (Gemini) track by resolving the remaining gaps identified in Audit 3. Specifically, wire token budgeting into the execution path, append truncation annotations, implement a robust fallback mechanism for Gemini execution failures, and add deterministic golden prompt tests.

## 2. Context & Rationale
The initial implementation of the Advanced Narrative Reporting feature established the foundational integration with the Gemini CLI. However, Audit 3 revealed critical missing pieces:
- The token budget (80% of 128k context window) was calculated but never enforced during Gemini execution, risking payload rejection.
- The required truncation annotation was absent.
- Gemini was not invoked with the required `analyze` argument.
- Fallback mechanisms for Gemini execution failures and missing CLI errors were incomplete.
- Deterministic golden prompt tests were missing, violating Phase 2 determinism principles.
- The `ask` command was not properly exposed with a `--narrative` flag and was incorrectly tied to a hardcoded `"summary"` query string.

## 3. Scope & Deliverables
- **`src/cli.rs` & `src/commands/ask.rs`**: Add `--narrative` flag mapping to `GeminiMode::Narrative`. Remove hardcoded `"summary"` query requirement for narrative generation.
- **`src/gemini/wrapper.rs`**: 
  - Invoke the Gemini CLI with the `analyze` argument.
  - Handle the missing Gemini CLI error with an actionable message.
- **`src/commands/ask.rs`**:
  - Wire in `packet.truncate_for_context()` before generating the prompt.
  - Append the `"[Packet truncated for Gemini submission]"` annotation when truncation occurs.
  - Save the impact packet as a fallback artifact when Gemini exits non-zero.
- **`tests/narrative_golden.rs`**: Introduce byte-for-byte deterministic prompt tests.

## 4. Functional Requirements

### 4.1. CLI Ergonomics
- Add `#[arg(long)] pub narrative: bool` to the `Ask` subcommand in `src/cli.rs`.
- In `src/commands/ask.rs`, map `--narrative` to `GeminiMode::Narrative`.
- Always use `NarrativeEngine::generate_risk_prompt` when in `Narrative` mode, instead of requiring the query to be exactly "summary".

### 4.2. Token Budgeting & Truncation
- Inside `src/commands/ask.rs` (or where the prompt is constructed), enforce a budget of 102.4k tokens (~409,600 characters).
- Call `latest_packet.truncate_for_context(409600)`.
- If truncation occurs, append `"\n\n[Packet truncated for Gemini submission]"` to the generated user prompt.

### 4.3. Gemini Execution Robustness
- In `src/gemini/wrapper.rs`, change `Command::new("gemini")` to `Command::new("gemini").arg("analyze")`.
- If spawning fails with `std::io::ErrorKind::NotFound`, return exactly: `"Gemini CLI not found. Install Gemini CLI to enable narrative summaries."`
- In `src/commands/ask.rs`, catch any execution error from `run_query()`. If it fails, write `latest_packet` to `.changeguard/reports/fallback-impact.json` (or similar fallback artifact), log a warning that a fallback was saved, and return the error.

### 4.4. Golden Prompt Tests
- Create `tests/narrative_golden.rs` or add tests to `src/gemini/narrative.rs`.
- Construct a deterministic `ImpactPacket` with multiple files, hotspots, and temporal couplings.
- Generate the narrative prompt.
- Assert that the output exactly matches a known "golden" string to ensure prompt construction stability.

## 5. Engineering Standards
- No production `unwrap()` or `expect()`.
- Error propagation must use idiomatic `miette::Result`.

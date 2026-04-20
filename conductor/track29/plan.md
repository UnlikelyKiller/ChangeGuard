## Plan: Track 29: Advanced Narrative Reporting (Gemini)

### Phase 1: Context Truncation & Token Management
- [ ] Task 1.1: Add `Estimator` logic in `src/gemini/wrapper.rs` to compute approximate token size of the impact packet (heuristic: 1 token ≈ 4 characters).
- [ ] Task 1.2: Implement priority-based truncation logic for the `ImpactPacket` (strip verification stdout, then unchanged file metadata, then oldest commits) to fit within 80% of the configured context window (default: 128k tokens).
- [ ] Task 1.3: Ensure `truncate_for_context` method safely preserves JSON/struct integrity and appends a `"Packet truncated for Gemini submission"` warning if any section was modified.
- [ ] Task 1.4: Write unit tests for truncation boundary and prioritization.

### Phase 2: Augmented Secret Redaction
- [ ] Task 2.1: Update `src/gemini/sanitize.rs` to include a static array of compiled regex patterns targeting common secrets (AWS, generic tokens, private keys). Add `regex` dependency if not already present.
- [ ] Task 2.2: Ensure redaction logic strictly operates on configuration values and strings, replacing them with `[REDACTED_SECRET]` without breaking JSON/TOML keys or syntax.
- [ ] Task 2.3: Write fixture tests verifying that configuration structure is preserved during aggressive redaction.

### Phase 3: Narrative Engine & Deterministic Prompts
- [ ] Task 3.1: Create `src/gemini/narrative.rs`.
- [ ] Task 3.2: Implement `NarrativeEngine::generate_prompt(packet: &ImpactPacket) -> Result<String>`.
- [ ] Task 3.3: Design a structured markdown prompt template that explicitly injects Phase 2 metrics (Temporal Coupling, Complexity Scores, Hotspots) to guide the Gemini model to synthesize multi-dimensional risks.
- [ ] Task 3.4: Write golden-file tests proving `generate_prompt` produces byte-for-byte identical string output given the same input `ImpactPacket`.

### Phase 4: Integration with Ask Command & Fallbacks
- [ ] Task 4.1: Update `src/commands/ask.rs` CLI argument parsing to add a `--narrative` flag or subcommand.
- [ ] Task 4.2: Wire the `ask` command to run the Token Estimator, Augmented Redaction, and Narrative Engine over the generated `ImpactPacket`.
- [ ] Task 4.3: Update `src/gemini/wrapper.rs` execution logic to run the Gemini CLI in `analyze` mode with the synthesized prompt. Handle missing CLI gracefully with a `miette::Result` error.
- [ ] Task 4.4: Ensure a fallback mechanism exists: if Gemini execution fails (non-zero exit code), capture stderr, report the failure as a diagnostic, and save the raw, redacted impact packet locally.

### Phase 5: Final Review & Documentation
- [ ] Task 5.1: Review all new error messages for actionable, deterministic phrasing per `docs/Engineering.md` (e.g., explaining why Gemini failed and what to do next).
- [ ] Task 5.2: Audit all modified/new files in `src/gemini/` and `src/commands/ask.rs` to ensure `unwrap()` and `expect()` are completely absent from production paths.
- [ ] Task 5.3: Verify End-to-End behavior of `changeguard ask --narrative`.
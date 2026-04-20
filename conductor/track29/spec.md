# Track 29: Advanced Narrative Reporting (Gemini)

## 1. Goal
Implement Phase 23 of the ChangeGuard Phase 2 plan. Enhance the existing `Ask` command to provide high-level, human-readable narrative summaries of risks and changes by feeding rich Phase 2 intelligence (Temporal, Complexity, Hotspots) to the Gemini CLI.

## 2. Context & Rationale
ChangeGuard gathers dense structural and historical data. While the JSON impact packets and CLI tables are excellent for machines and quick glances, complex changes require a synthesized "Executive Summary." This track leverages the Gemini CLI to synthesize the multi-dimensional risk data into an actionable narrative.

We must adhere strictly to the Engineering Principles: no silent fallbacks, strict determinism in prompt construction, and robust error visibility.

## 3. Scope & Deliverables
- **`src/gemini/narrative.rs`**: Core engine for formatting the impact packet into a structured prompt and delegating to the Gemini CLI.
- **`src/gemini/wrapper.rs` (updates)**: Enhance with a token budget estimator to gracefully truncate large impact packets.
- **`src/gemini/sanitize.rs` (updates)**: Augment secret redaction with static regex patterns (inspired by gitleaks, but native). Redaction must preserve configuration syntax (JSON/TOML).
- **`src/commands/ask.rs` (updates)**: Integrate a `--narrative` flag or subcommand to trigger this flow.

## 4. Functional Requirements

### 4.1. Narrative Generation Engine (`src/gemini/narrative.rs`)
- Synthesizes an "Executive Summary" markdown document.
- Ingests the Phase 2 Impact Packet (including temporal coupling, complexity scores, and hotspots).
- Formats a deterministic, structured prompt instructing Gemini to analyze the multi-dimensional risk.
- Must execute the Gemini CLI in `analyze` mode.

### 4.2. Token Budget Estimator (`src/gemini/wrapper.rs`)
- Introduce an estimator to prevent exceeding Gemini's sequence length.
- Threshold: Truncate if the packet exceeds 80% of the configured context window (default: 128k tokens). Estimate 1 token ≈ 4 characters.
- Truncation strategy: Priority-based. Strip verification stdout first, then unchanged file metadata, then oldest commit histories, until the packet fits.
- Append a `"Packet truncated for Gemini submission"` annotation to the prompt/summary if truncation occurred.

### 4.3. Augmented Secret Redaction (`src/gemini/sanitize.rs`)
- Embed a static list of secret-detecting regex patterns (e.g., AWS keys, generic API keys, private keys). DO NOT depend on `gitleaks`.
- Redaction MUST operate on values only, preserving the structural integrity of JSON, TOML, and YAML payloads.
- All secrets detected must be replaced with `[REDACTED_SECRET]`.

### 4.4. CLI Integration (`src/commands/ask.rs`)
- Add `changeguard ask --narrative` (or a specific subcommand if appropriate).
- Output the resulting Markdown summary directly to stdout (or designated report file).

## 5. Error Handling & Edge Cases
- **Missing Gemini CLI**: Return an explicit, actionable `miette::Result` error: `"Gemini CLI not found. Install Gemini CLI to enable narrative summaries."`
- **Gemini CLI Failure**: If the command exits non-zero, capture `stderr`, report it via `miette`, and save the raw impact packet as a fallback artifact.
- **Overly Large Inputs**: Covered by the Token Budget Estimator. Ensure truncation itself doesn't corrupt JSON structure before sending to Gemini.
- **No `unwrap()` / `expect()`**: Propagate errors idiomatically.

## 6. Determinism & Testing
- **Prompt Determinism**: Given the same impact packet, the resulting string prompt MUST be byte-for-byte identical. Use golden-file tests.
- **Redaction Integrity**: Fixture tests must prove that JSON/TOML keys are preserved and only sensitive values are redacted.
- **Token Estimation**: Unit tests for the character/token threshold and prioritized truncation.
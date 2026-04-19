# Specification: Track 14 — Critical Safety Fixes

## Overview
Address the 4 critical and high-severity correctness/security issues identified in the audit: secret redaction, verification planning, silent error suppression, and production `unwrap()`.

## 1. Secret Redaction (`src/impact/redact.rs` and `src/gemini/sanitize.rs`)
**Priority: CRITICAL** — Plan Section 7.2 requires redaction of likely secrets.

### `src/impact/redact.rs`
- Define `pub fn redact_secrets(packet: &mut ImpactPacket) -> Vec<String>` that scans:
  - Changed file paths matching `.env*`, `credentials*`, `*.pem`, `*.key`
  - File contents for patterns matching common secret formats (AWS keys, GitHub tokens, private keys, generic long base64/alpha strings after `=` or `:` in env-like files)
- Replace matched content with `[REDACTED]` markers in the packet's `ChangedFile` entries
- Return list of redaction reasons for auditability
- Use the `regex` crate (already in Cargo.toml but unused)

### `src/gemini/sanitize.rs`
- Define `pub fn sanitize_prompt(prompt: &str) -> String` that strips or masks likely secrets from the combined prompt text before piping to Gemini
- Reuse the same regex patterns from `redact.rs`
- Add a size limit: truncate prompts exceeding a configurable byte threshold (default 256KB) with a warning annotation

### Integration
- `commands/impact.rs`: call `redact_secrets(&mut packet)` before `write_impact_report` and `save_packet`
- `commands/ask.rs`: call `sanitize_prompt` on the combined prompt before `run_query`
- Add `redact` module to `src/impact/mod.rs`
- Add `sanitize` module to `src/gemini/mod.rs`

## 2. Verification Planning (`src/verify/plan.rs`)
**Priority: CRITICAL** — Phase 11 core requirement; `required_verifications` is defined but never consumed.

### `src/verify/mod.rs`
- Public module root

### `src/verify/plan.rs`
- Define `VerificationPlan` struct with a list of `VerificationStep` (command, timeout, description)
- Define `pub fn build_plan(packet: &ImpactPacket, rules: &Rules) -> VerificationPlan`
  - Merge global `required_verifications` from rules
  - Merge path-specific `required_verifications` from matching `PathRule` entries
  - Deduplicate commands
  - Sort deterministically
- Deterministic: same packet + same rules = same plan, always

### Integration
- `commands/verify.rs`: when no `--command` is provided, build a plan from the latest packet + rules instead of defaulting to `cargo test`
- Register `verify` module in `src/lib.rs`

## 3. Silent Error Suppression Fix
**Priority: CRITICAL** — Engineering.md: "invalid config never causes silent fallback without warning."

### `commands/impact.rs` line 38-39
- Replace `if let Ok(rules)` with explicit match. On `Err(e)`, emit `tracing::warn!` and print a warning to the user that risk analysis was skipped, including the error message.
- Replace `let _ = analyze_risk(...)` with proper error handling. On failure, emit `tracing::warn!` and print a warning.

### `commands/impact.rs` line 56-58
- Replace `if let Ok(storage)` with explicit match. On `Err(e)`, emit `tracing::warn!` and print a warning that the SQLite ledger was not updated. The `ask` command depends on this.

## 4. Production `unwrap()` Fix
**Priority: HIGH** — Engineering.md: "no `unwrap`/`expect` in production paths."

### `src/index/languages/python.rs:38`
- Replace `capture.node.parent().unwrap()` with a conditional check:
  ```rust
  if let Some(parent) = capture.node.parent() {
      // check parent kind
  }
  ```
- If `parent()` returns `None`, treat the symbol as non-underscore (default to public convention).

## Verification
- Unit tests for `redact_secrets` with fixtures containing fake API keys, tokens, `.env` content
- Unit tests for `sanitize_prompt` verifying secrets are stripped
- Unit tests for `build_plan` verifying determinism and deduplication
- Integration test `tests/cli_impact.rs` verifying warning output when rules/DB fail
- `cargo test -j 1 -- --test-threads=1`
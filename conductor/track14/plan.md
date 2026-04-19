## Plan: Track 14 — Critical Safety Fixes

### Phase 1: Secret Redaction
- [ ] Task 14.1: Create `src/impact/redact.rs`. Define `Redaction` struct. Implement shared secret-matching regex patterns (AWS keys, GitHub tokens, Google API keys, OpenAI keys, private key blocks, generic env-var secrets). Include Shannon entropy check (>=4.5 bits/char, >=20 chars) for generic high-entropy detection in `.env*` files. Implement `redact_secrets(&mut ImpactPacket) -> Vec<Redaction>`.
- [ ] Task 14.2: Create `src/gemini/sanitize.rs`. Define `SanitizeResult` struct. Implement `sanitize_prompt(prompt, max_bytes) -> SanitizeResult` reusing patterns from `redact.rs`. Truncate at paragraph boundary (double newline), fallback to last newline within 10% margin, then append truncation annotation. Default max_bytes: 256KB.
- [ ] Task 14.3: Add `redact` to `src/impact/mod.rs` and `sanitize` to `src/gemini/mod.rs`.
- [ ] Task 14.4: Wire `redact_secrets` into `commands/impact.rs` before `write_impact_report` and `save_packet`. Wire `sanitize_prompt` into `commands/ask.rs` before `run_query`. Pass `gemini.timeout_secs` from config (default 120s) to `run_query`.
- [ ] Task 14.5: Write unit tests for `redact_secrets` with fixtures containing fake secrets. Write unit tests for `sanitize_prompt` (secret stripping + paragraph-boundary truncation). Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 2: Verification Planning
- [ ] Task 14.6: Create `src/verify/mod.rs` and `src/verify/plan.rs`. Define `VerificationStep`, `VerificationPlan`, and `build_plan(packet, rules) -> VerificationPlan`.
- [ ] Task 14.7: Implement plan generation: merge global + path-specific `required_verifications`, deduplicate by exact command string, sort alphabetically. Implement fallback to `cargo test -j 1 -- --test-threads=1` when no rules or no required_verifications.
- [ ] Task 14.8: Register `verify` module in `src/lib.rs`. Update `commands/verify.rs` to use `build_plan` when no `--command` is given. Load latest packet from SQLite for build_plan input.
- [ ] Task 14.9: Write unit tests for `build_plan` (determinism, deduplication, path-rule merging, empty-rules fallback). Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Silent Error Suppression Fix
- [ ] Task 14.10: In `commands/impact.rs`, replace `if let Ok(rules)` with match. On error, `tracing::warn!` and print user-facing warning using `warning_marker()`. Handle `analyze_risk` failure similarly.
- [ ] Task 14.11: In `commands/impact.rs`, replace `if let Ok(storage)` with match. On error, `tracing::warn!` and print warning about `ask` command dependency.
- [ ] Task 14.12: In `src/gemini/prompt.rs`, replace `unwrap_or_else(|_| "{}")` with proper error message including the serialization error.
- [ ] Task 14.13: Create `tests/cli_impact.rs` verifying warning output on rules/DB failure paths. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 4: Production `unwrap()`/`expect()` Fix + Gemini Timeout
- [ ] Task 14.14: In `src/index/languages/python.rs`, replace `capture.node.parent().unwrap()` with `if let Some(parent)`. Handle `None` case (default to public).
- [ ] Task 14.15: In `src/commands/impact.rs:88`, replace `.expect()` with `.unwrap_or_else(|_| ProgressStyle::default_bar())`.
- [ ] Task 14.16: In `src/gemini/mod.rs:12-13`, replace `.expect()` with `.unwrap_or_else(|_| ProgressStyle::default_spinner())`.
- [ ] Task 14.17: In `src/gemini/mod.rs`, replace `child.wait_with_output()` with `wait_timeout`-based bounded wait. Add `gemini_timeout_secs: u64` parameter to `run_query`. Kill child process on timeout and return clear error. Default timeout: 120s from config, overridable.
- [ ] Task 14.18: Verify no remaining `unwrap()`/`expect()` in production paths (search entire `src/` for `.unwrap()` and `.expect(` outside of test modules). Run `cargo test -j 1 -- --test-threads=1`.

### Phase 5: Final Verification
- [ ] Task 14.19: `cargo clippy --all-targets --all-features` and `cargo fmt --check`. Resolve any warnings.
- [ ] Task 14.20: Full suite `cargo test -j 1 -- --test-threads=1`.
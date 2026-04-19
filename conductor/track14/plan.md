## Plan: Track 14 — Critical Safety Fixes

### Phase 1: Secret Redaction
- [ ] Task 14.1: Create `src/impact/redact.rs`. Define secret-matching regex patterns (AWS keys, GitHub tokens, private key blocks, generic env-var secrets). Implement `redact_secrets(&mut ImpactPacket) -> Vec<String>`.
- [ ] Task 14.2: Create `src/gemini/sanitize.rs`. Implement `sanitize_prompt(prompt: &str) -> String` reusing patterns from `redact.rs`. Add a 256KB size limit with truncation annotation.
- [ ] Task 14.3: Add `redact` to `src/impact/mod.rs` and `sanitize` to `src/gemini/mod.rs`.
- [ ] Task 14.4: Wire `redact_secrets` into `commands/impact.rs` before `write_impact_report` and `save_packet`. Wire `sanitize_prompt` into `commands/ask.rs` before `run_query`.
- [ ] Task 14.5: Write unit tests for `redact_secrets` with fixtures containing fake secrets. Write unit tests for `sanitize_prompt`. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 2: Verification Planning
- [ ] Task 14.6: Create `src/verify/mod.rs` and `src/verify/plan.rs`. Define `VerificationPlan`, `VerificationStep`, and `build_plan(packet, rules) -> VerificationPlan`.
- [ ] Task 14.7: Implement plan generation: merge global + path-specific `required_verifications`, deduplicate, sort deterministically.
- [ ] Task 14.8: Register `verify` module in `src/lib.rs`. Update `commands/verify.rs` to use `build_plan` when no `--command` is given.
- [ ] Task 14.9: Write unit tests for `build_plan` (determinism, deduplication, path-rule merging). Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 3: Silent Error Suppression Fix
- [ ] Task 14.10: In `commands/impact.rs`, replace `if let Ok(rules)` with match. On error, `tracing::warn!` and print user-facing warning. Handle `analyze_risk` failure similarly.
- [ ] Task 14.11: In `commands/impact.rs`, replace `if let Ok(storage)` with match. On error, `tracing::warn!` and print warning about `ask` command dependency.
- [ ] Task 14.12: Create `tests/cli_impact.rs` verifying warning output on rules/DB failure paths. Verify with `cargo test -j 1 -- --test-threads=1`.

### Phase 4: Production `unwrap()` Fix
- [ ] Task 14.13: In `src/index/languages/python.rs`, replace `capture.node.parent().unwrap()` with `if let Some(parent)`. Handle `None` case gracefully.
- [ ] Task 14.14: Verify no remaining `unwrap()`/`expect()` in production paths. Run `cargo test -j 1 -- --test-threads=1`.

### Phase 5: Final Verification
- [ ] Task 14.15: `cargo clippy --all-targets --all-features` and `cargo fmt --check`. Resolve any warnings.
- [ ] Task 14.16: Full suite `cargo test -j 1 -- --test-threads=1`.
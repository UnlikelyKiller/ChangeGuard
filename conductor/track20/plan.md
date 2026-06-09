## Plan: Track 20 — Determinism and Error Visibility Hardening

### Phase 1: Rule Validation on Load
- [ ] Task 20.1: Update `src/policy/load.rs` to call `validate_rules()` after TOML parse and before returning.
- [ ] Task 20.2: Add tests for invalid override globs and invalid protected-path globs failing through `load_rules()`.
- [ ] Task 20.3: Preserve the existing missing-file default path for `rules.toml`.

### Phase 2: Command Fallback Removal
- [ ] Task 20.4: Replace `load_config(...).unwrap_or_default()` in `commands/ask.rs` with explicit error handling that still allows the loader’s missing-file default behavior.
- [ ] Task 20.5: Replace `load_config(...).unwrap_or_default()` in `commands/watch.rs` with explicit error handling that still allows the loader’s missing-file default behavior.
- [ ] Task 20.6: Replace `load_rules(...).unwrap_or_default()` in `commands/verify.rs` auto-plan flow with explicit error handling.
- [ ] Task 20.7: Decide and document whether any fallback remains; if so, make it user-visible and test-covered.

### Phase 3: Partial-Analysis Visibility
- [ ] Task 20.8: Extend the impact packet model with structured partial-analysis status or warning fields.
- [ ] Task 20.9: Update `commands/impact.rs` so symbol/import/runtime extraction failures are captured explicitly rather than dropped.
- [ ] Task 20.10: Update `output::human` to summarize partial analysis without making normal output noisy.
- [ ] Task 20.11: Ensure warning/status collections are sorted and deduplicated before persistence/output.
- [ ] Task 20.12: Preserve packet compatibility with additive fields where possible; if not possible, bump and document the schema version.

### Phase 4: Tests
- [ ] Task 20.13: Add tests covering invalid config/rules behavior in `ask`, `watch`, and `verify`.
- [ ] Task 20.14: Add impact-generation tests for unreadable files, unsupported files, and parser/extractor failures producing explicit status.
- [ ] Task 20.15: Add deterministic-ordering tests for warnings/status entries.
- [ ] Task 20.16: Add packet compatibility/schema tests covering any new fields or version bumps.

### Phase 5: Final Verification
- [ ] Task 20.17: `cargo fmt --check`
- [ ] Task 20.18: `cargo clippy --all-targets --all-features`
- [ ] Task 20.19: `cargo test -j 1 -- --test-threads=1`

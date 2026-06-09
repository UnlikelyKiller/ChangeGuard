## Plan: Track M7-5 — CI Pipeline Self-Awareness

### Phase 1: CI Config Detection
- [ ] Task 1.1: Extend `src/index/ci_gates.rs` with `is_ci_config_changed()`.
- [ ] Task 1.2: Detect known CI config patterns.
- [ ] Task 1.3: Detect non-standard CI paths (`.github/**`, `.ci/**`, `ci/**`).
- [ ] Task 1.4: Implement generated CI file detection (`is_generated_ci_file()`).
- [ ] Task 1.5: Implement pre-commit hook detection (`detect_pre_commit_changes()`).
- [ ] Task 1.6: Write test: `.github/workflows/ci.yml` → CI risk reason.
- [ ] Task 1.7: Write test: `Jenkinsfile` → CI risk reason.
- [ ] Task 1.8: Write test: `generated-ci.yml` with `# auto-generated` → informational only.
- [ ] Task 1.9: Write test: `.pre-commit-config.yaml` → pre-commit reason.

### Phase 2: Risk Enrichment
- [ ] Task 2.1: Implement risk weighting: CI-only=3, CI+source=5, CI+deploy=tier+1.
- [ ] Task 2.2: Wire into `execute_impact()` enrichment pipeline.
- [ ] Task 2.3: Write test: CI config + source co-change → Medium elevation.
- [ ] Task 2.4: Write test: CI alone → Low elevation.
- [ ] Task 2.5: Write test: CI + deploy → escalated tier.
- [ ] Task 2.6: Write test: `[coverage.ci_self_awareness].enabled = false` → no detection.
- [ ] Task 2.7: Write test: `Makefile` with `test` target → CI risk reason.
- [ ] Task 2.8: Write test: `Makefile` without CI targets → no risk reason.

### Phase 3: Final Validation
- [ ] Task 3.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Task 3.2: Run `cargo test index::ci_gates` — all tests pass.
- [ ] Task 3.3: Run full `cargo test` — no regressions.

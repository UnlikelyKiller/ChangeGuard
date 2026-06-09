# Track Y1 Plan: Integration Test Coverage for Untested Command Surfaces

## Phase 1 — Config Surface Tests (3 tests)
- [ ] 1. Write `tests/integration/cli_config.rs::test_config_verify_default`: init temp repo, create `.changeguard/` with default config, run `config verify`, assert exit 0 and JSON parseable.
- [ ] 2. Write `test_config_view_shows_values`: run `config view`, assert key config keys appear in output.
- [ ] 3. Write `test_config_schema_output`: run `config schema`, assert JSON or table output is non-empty.

## Phase 2 — Codebase Surface Tests (5 tests)
- [ ] 4. Write `tests/integration/cli_surfaces.rs::test_endpoints_json`: after `index --incremental --analyze-graph`, run `endpoints --json`, assert JSON array output (may be empty).
- [ ] 5. Write `test_data_models_impact_changed`: run `data-models impact --changed` on clean tree, assert empty or graceful "no changes" message.
- [ ] 6. Write `test_observability_coverage_json`: run `observability coverage --json`, assert `[]` or valid JSON.
- [ ] 7. Write `test_security_boundaries_human`: run `security boundaries`, assert non-crash and expected empty-state hint.
- [ ] 8. Write `test_services_diff`: run `services diff`, assert non-crash.

## Phase 3 — Remaining Surface Tests (3 tests)
- [ ] 9. Write `tests/integration/cli_dead_code.rs::test_dead_code_basic`: run `dead-code --threshold 0.9`, assert exit 0 and JSON or empty-state output.
- [ ] 10. Write `tests/integration/cli_viz.rs::test_viz_generates_html`: run `viz --output %T/out.html`, assert file exists and contains `<!DOCTYPE html>`.
- [ ] 11. Write `tests/integration/cli_update.rs::test_update_dry_run`: run `update --dry-run`, assert exit 0 and no persistent state changes.

## Phase 4 — Advanced Surface Tests (2 tests)
- [ ] 12. Write `tests/integration/cli_federate.rs::test_federate_scan_no_remotes`: run `federate scan`, assert graceful empty or "no remotes" message.
- [ ] 13. Write `tests/integration/cli_audit.rs::test_audit_basic`: run `audit --limit 5`, assert exit 0 and valid output.

## Phase 5 — Green + Cleanup
- [ ] 14. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 15. Run `cargo nextest run --test integration` — all pass.
- [ ] 16. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 17. Run `cargo fmt --all -- --check` — clean.
- [ ] 18. Update `conductor/conductor.md` status to Completed.
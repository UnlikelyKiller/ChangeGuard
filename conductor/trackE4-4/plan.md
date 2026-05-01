## Plan: Track E4-4 Runtime Usage in Risk Scoring

### Phase 1: Common Env Var and Framework Convention Constants
- [ ] Task 1.1: Define `COMMON_ENV_VARS` constant list in `src/impact/analysis.rs`: `PATH`, `HOME`, `USER`, `LANG`, `SHELL`, `TERM`, `PWD`.
- [ ] Task 1.2: Define `FRAMEWORK_CONVENTION_KEYS` constant list in `src/impact/analysis.rs`: `server.port`, `logging.level`, `database.url`, `server.host`, `server.timeout`, `log.level`.
- [ ] Task 1.3: Write unit tests verifying that common env vars are correctly excluded from new-dependency detection.
- [ ] Task 1.4: Write unit tests verifying that framework convention config keys receive reduced risk weight (+5 instead of +10).

### Phase 2: Risk Scoring Integration - New Env Var Dependencies
- [ ] Task 2.1: Extend `analyze_risk()` in `src/impact/analysis.rs` to collect `runtime_usage.env_vars` from each changed file and compare against `env_schema`.
- [ ] Task 2.2: For each env var in a changed file that is NOT in `env_schema`, add +20 risk weight and "New environment variable dependency: X" to `risk_reasons`.
- [ ] Task 2.3: For each env var in a changed file that IS in `env_schema` (with a non-empty `default_value`), add +10 risk weight instead of +20.
- [ ] Task 2.4: Exclude common env vars (`COMMON_ENV_VARS`) from the new-dependency check.
- [ ] Task 2.5: Write test: a file with `std::env::var("NEW_VAR")` that is not in `env_schema` receives +20 risk weight and a "New environment variable dependency" risk reason.
- [ ] Task 2.6: Write test: a file with `std::env::var("PATH")` does NOT receive the +20 risk weight (common env var exclusion).
- [ ] Task 2.7: Write test: a file with `std::env::var("DATABASE_URL")` where `DATABASE_URL` exists in `env_schema` with a default value receives +10 risk weight (reduced, not +20).

### Phase 3: Risk Scoring Integration - Env Var Reference Changes
- [ ] Task 3.1: Extend `analyze_risk()` to compare the current file's `runtime_usage.env_vars` count against the previous version.
- [ ] Task 3.2: When env var references are added or removed (count changes), add +10 risk weight and "Environment variable references changed in X" to `risk_reasons`.
- [ ] Task 3.3: Write test: a file that adds a new env var reference (increasing the count) receives +10 risk weight and an "Environment variable references changed" risk reason.
- [ ] Task 3.4: Write test: a file that removes an env var reference (decreasing the count) receives +10 risk weight.

### Phase 4: Risk Scoring Integration - Config Key Reference Changes
- [ ] Task 4.1: Extend `analyze_risk()` to compare the current file's `runtime_usage.config_keys` count against the previous version.
- [ ] Task 4.2: When config key references are added or removed (count changes), add +10 risk weight and "Configuration key references changed in X" to `risk_reasons`.
- [ ] Task 4.3: For framework convention config keys (`FRAMEWORK_CONVENTION_KEYS`), use reduced weight (+5 instead of +10).
- [ ] Task 4.4: Write test: a file that adds a new config key reference receives +10 risk weight and a "Configuration key references changed" risk reason.
- [ ] Task 4.5: Write test: a file that adds `server.port` config key reference receives +5 risk weight (reduced, framework convention).

### Phase 5: Verification Prediction Integration
- [ ] Task 5.1: Modify `src/verify/predict.rs` to include env-var-based prediction reasons when a changed file introduces new env var dependencies.
- [ ] Task 5.2: Add prediction reason: "New env var dependency: X" for each new env var dependency found in the changed file.
- [ ] Task 5.3: Add warning: "Removed env var usage: X" for each env var reference that was removed from the changed file.
- [ ] Task 5.4: Write test: changing a file that introduces a new env var dependency produces a prediction reason mentioning the env var.
- [ ] Task 5.5: Write test: changing a file that removes an env var reference produces a warning about removed usage.

### Phase 6: JSON Report Extension
- [ ] Task 6.1: Ensure `runtime_usage` field on `ChangedFile` is included in the JSON serialization output of `ImpactPacket`.
- [ ] Task 6.2: Add `#[serde(skip_serializing_if = "Vec::is_empty")]` or equivalent to avoid unnecessary empty fields in JSON output.
- [ ] Task 6.3: Ensure `env_var_deps` field on `ImpactPacket` is included in the JSON serialization output (may overlap with Track E4-3; coordinate to avoid duplication).
- [ ] Task 6.4: Write test: `impact --json` output includes `runtime_usage` data with `env_vars` and `config_keys` for changed files that have runtime usage.
- [ ] Task 6.5: Write test: `impact --json` output includes `env_var_deps` when new env var dependencies are detected.
- [ ] Task 6.6: Write test: backward compatibility - JSON consumers that don't expect `runtime_usage` or `env_var_deps` are not broken.

### Phase 7: Graceful Degradation and Edge Cases
- [ ] Task 7.1: Implement graceful degradation: if `runtime_usage` extraction fails for a file, skip risk weight for that file and log a warning. Do not crash.
- [ ] Task 7.2: Write test: a file with empty `runtime_usage` (no env vars or config keys) does not receive runtime-usage-based risk weights.
- [ ] Task 7.3: Write test: `runtime_usage` extraction failure does not crash the `impact` command.
- [ ] Task 7.4: Write test: the entire `impact` pipeline still works correctly when `env_schema` table is empty (no prior `index` run).

### Phase 8: Final Validation
- [ ] Task 8.1: Run full test suite (`cargo test`) and verify no regressions in existing `impact`, `hotspots`, `verify`, or `ledger` tests.
- [ ] Task 8.2: Run `changeguard impact` on a fixture repo with env var and config key changes, and verify risk weights and risk reasons appear in the output.
- [ ] Task 8.3: Run `changeguard impact --json` and verify `runtime_usage` and `env_var_deps` appear in the JSON output.
- [ ] Task 8.4: Run `changeguard verify` on a fixture repo and verify env-var-based prediction reasons appear in the verification plan.
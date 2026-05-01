# Specification: Track E4-4 Runtime Usage in Risk Scoring

## Overview

Implement the fourth track of Phase E4 (Safety Context) from `docs/expansion-plan.md`. This track wires the already-extracted `runtime_usage` data (environment variables, config keys) into the risk scoring and verification prediction systems, and adds `runtime_usage` to the JSON impact report output.

## Motivation

The `runtime_usage` field on `ChangedFile` is populated during `impact` runs but has zero effect on risk scoring or prediction. Environment variable references and config key accesses represent runtime dependencies that, when changed, can cause production failures. This track closes the gap by making `runtime_usage` a first-class participant in risk analysis and verification planning.

## Components

### 1. Risk Scoring Integration (`src/impact/analysis.rs`)

Extend `analyze_risk()` to incorporate `runtime_usage` data into risk weight calculations:

**New env var dependencies (+20 risk weight, capped):**
- When a file's `runtime_usage.env_vars` contains entries that are NOT in the project's `env_declarations` table (from Track E4-3), add risk weight in the **Runtime/Config** category.
- Add "New environment variable dependency: X" to `risk_reasons` for each new env var.
- Common env vars (`PATH`, `HOME`, `USER`, `LANG`, `SHELL`, `TERM`, `PWD`) are excluded from this check.
- **Risk category:** New env var dependencies contribute up to 20 points to the Runtime/Config category (max 25 points total for the category).

**Env var reference changes (+10 risk weight, capped):**
- When a file's `runtime_usage.env_vars` count changes between the current and previous version (env vars added or removed), add risk weight in the **Runtime/Config** category.
- Add "Environment variable references changed in X" to `risk_reasons`.
- **Risk category:** Env var reference changes contribute up to 10 points to the Runtime/Config category.

**Config key reference changes (+10 risk weight, capped):**
- When a file's `runtime_usage.config_keys` count changes between the current and previous version (config keys added or removed), add risk weight in the **Runtime/Config** category.
- Add "Configuration key references changed in X" to `risk_reasons`.
- **Risk category:** Config key reference changes contribute up to 10 points to the Runtime/Config category.

**Framework convention keys (reduced weight):**
- Config keys that are framework conventions (`server.port`, `logging.level`, `database.url`) receive +5 instead of +10, as these are common and less risky.
- This is a heuristic and may be expanded in future phases.
- **Risk category:** Framework convention keys contribute up to 5 points to the Runtime/Config category.

**Category cap:** The Runtime/Config category has a maximum of 25 points per expansion plan Section 4.2. All Runtime/Config risk reasons (new env vars, changed env var references, changed config key references, framework convention keys) are summed and then capped at 25.

### 2. Verification Prediction Integration (`src/verify/predict.rs`)

Extend verification prediction to include env-var-based and config-key-based prediction reasons:

**New env var dependencies:**
- When a changed file introduces new env var dependencies (not in `env_declarations`), add a prediction reason: "New env var dependency: X".
- These predictions appear alongside test-mapping, temporal coupling, and structural import predictions.

**Removed env var references:**
- When a changed file removes env var references, add a warning: "Removed env var usage: X".
- This is informational and does not affect prediction ordering.

### 3. JSON Report Extension (`src/impact/packet.rs`)

Add `runtime_usage` to the serialized `ImpactPacket` JSON output:

- The `runtime_usage` field on `ChangedFile` is already populated during `impact` but is not included in the serialized JSON output.
- Add `runtime_usage` to the `ChangedFile` serialization so that it appears in `impact --json` output.
- The `runtime_usage` field contains `env_vars: Vec<String>` and `config_keys: Vec<String>`.
- Ensure backward compatibility: the field should be present in new output but consumers that don't expect it should not break (use `#[serde(default)]` or `#[serde(skip_serializing_if = "Vec::is_empty")]`).

### 4. ImpactPacket Extension (`src/impact/packet.rs`)

Add `env_var_deps` to `ImpactPacket` (if not already added by Track E4-3):

```rust
#[serde(default)]
pub env_var_deps: Vec<EnvVarDep>,
```

Where `EnvVarDep` includes `var_name`, `file_id` (referencing `project_files`), `source`, `is_new`, `confidence`, and `evidence`. This overlaps with Track E4-3; if both tracks are implemented, ensure only one definition exists.

**Dependency on Track E4-3:** This track depends on E4-3's two-table design (`env_declarations` + `env_references`). The `env_schema` table name no longer exists — it has been split into `env_declarations` for declarations from config files and `env_references` for references from source code. When checking for new env var dependencies, query `env_declarations` (not `env_schema`).

## Constraints & Guidelines

- **Graceful degradation**: If `runtime_usage` extraction fails for a file, skip risk weight for that file. Do not propagate extraction errors into risk scoring failures.
- **Common env var exclusion**: `PATH`, `HOME`, `USER`, `LANG`, `SHELL`, `TERM`, `PWD` are too common to be meaningful as new dependencies. Exclude them from the +20 risk weight and from "new dependency" warnings.
- **Framework convention keys**: `server.port`, `logging.level`, `database.url`, and similar framework conventions receive reduced weight (+5 instead of +10) because they are standard and less likely to cause failures.
- **Risk category**: All runtime/config risk weights (new env vars, changed env var references, changed config key references, framework convention keys) fall within the **Runtime/Config category (max 25 points)**. New env vars: up to 20 points. Changed references: up to 10 points. Framework convention keys: up to 5 points. All within the 25-point cap.
- **Dependency on E4-3's table design**: This track queries `env_declarations` (not `env_schema`) for checking whether env var references are new. The `env_schema` table name no longer exists — it has been split into `env_declarations` and `env_references` by Track E4-3.
- **No false confidence**: Do not flag env var additions as risks if the env var already exists in `env_schema` with a default value and is not marked as `required`.
- **TDD Requirement**: Write or update tests for risk weight application, prediction reasons, and JSON report serialization.
- **No performance regression**: The runtime_usage risk scoring must not add more than 2% overhead to the `impact` command.
- **Backward-compatible JSON**: The `runtime_usage` field in the JSON output must not break existing consumers. Use `#[serde(skip_serializing_if)]` or similar to avoid unnecessary empty fields.

## Edge Cases

- **`runtime_usage` extraction failure**: Degrade gracefully. Skip risk weight for that file. Log a warning. Do not crash.
- **Very common env vars** (`PATH`, `HOME`, etc.): Skip in risk scoring entirely. These are system env vars that are always present.
- **Config keys that are framework conventions** (`server.port`, `logging.level`): Use reduced weight (+5 instead of +10).
- **Env vars with default values in `env_declarations`**: If an env var already exists in `env_declarations` with a `default_value_redacted` of `HAS_DEFAULT`, it is less risky. Use reduced weight (+10 instead of +20) for new references to already-documented env vars.
- **Empty `runtime_usage`**: If `runtime_usage` is empty for all changed files, skip all runtime-usage-based risk weights and predictions. This is the common case for repos without env var or config key usage.

## Acceptance Criteria

- `changeguard impact` gives elevated risk weight (+20) to files with new env var dependencies not in `env_schema`.
- `changeguard impact` gives elevated risk weight (+10) to files with changed env var references.
- `changeguard impact` gives elevated risk weight (+10) to files with changed config key references.
- `changeguard impact` gives reduced weight (+5) for framework convention config keys.
- `changeguard verify` includes env-var-based prediction reasons in verification plans.
- `impact --json` output includes `runtime_usage` data in `ChangedFile` entries.
- Common env vars (`PATH`, `HOME`, etc.) are excluded from risk scoring.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M18 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E4 tables for fixture repos
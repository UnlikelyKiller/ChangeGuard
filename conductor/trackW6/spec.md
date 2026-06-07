# Track W6 Spec: Config and Environment Variable Ownership

## Background

Config and env var tracking currently scores 8/10. ChangeGuard tracks declarations and references well, but ownership, requiredness, environment scope, and secret metadata are incomplete.

## Objective

Raise config tracking to 9/10 by making env/config schema metadata first-class and linking config keys to services, endpoints, deploy manifests, tests, ADRs, and runtime risk.

## Proposed Design

1. Add metadata fields for required, optional, default, secret, owner, environment, rotation policy, provider, rollout notes, and service scope.
2. Merge inferred declarations/references with optional schema overlays.
3. Detect config key removals, default changes, requiredness changes, secret exposure, example drift, and environment-only config drift.
4. Add `config schema` and `config diff` commands with stable JSON output.
5. Add policy support for protected or required env vars per service.

## Critical Files

| File | Expected work |
|---|---|
| `src/index/env_schema.rs` | Extend env declaration/reference model |
| `src/index/runtime_usage.rs` | Link runtime usage to config schema |
| `src/impact/enrichment/environment.rs` | Add ownership and requiredness risk |
| `src/config/model.rs` | Add schema overlay configuration |
| `src/commands/config.rs` | Add schema/diff command surface |

## Definition of Done

- Requiredness, default, owner, secret status, and environment scope are visible in human and JSON output.
- Inferred and declared config facts produce deterministic conflict reporting.
- Secret and required-env changes raise appropriate risk without printing secret values.
- Target score after completion: 9/10.

# Track Y5: CLI UX Consistency — Category Enum, Dry-Run Plan, Env-Var Identity

**Status:** Planned  
**Milestone:** Y — CLI Reliability & UX Hardening  
**Priority:** Medium

## Objective

Fix three medium-friction UX issues identified in the command audit that degrade daily developer experience but do not produce crashes.

## Problem Statement

### Issue 1: `ledger start --category` is a free-string
`changeguard ledger start` accepts `--category` as a raw `String`, while `ledger atomic` accepts a `Category` enum. This means:
- No tab-completion for `start --category`.
- Invalid category strings are silently accepted and stored in the DB.
- The `enum` approach in `atomic` uses `clap::ValueEnum` which provides validation at the CLI layer.
- Fix: Change `--category` in `LedgerStartArgs` to use the same `Category` enum.

### Issue 2: `verify --dry-run` shows no plan
`changeguard verify --dry-run` prints nothing about what would have been run. Users see a silent exit 0. It should print the verification plan (list of commands to execute, predicted failures from the Predictor), matching user expectations from tools like `terraform plan`.

### Issue 3: Risk analysis tracks env-var cardinality, not identity
The `RuntimeUsageDelta` model tracks counts of env vars, config keys, and service URLs — not which specific ones changed. Replacing `DATABASE_URL` with `REDIS_URL` (1-to-1 cardinality) produces no risk signal even though runtime behavior changes completely. Fix: Compare sets of env-var names between old and new snapshots, flagging replacements even when cardinality is unchanged.

## Acceptance Criteria

1. `ledger start --category` uses the same `Category` enum as `ledger atomic` (with `clap::ValueEnum` derive, tab-completion, validation).
2. `verify --dry-run` prints the full verification plan (command list, timeout, description, predicted failures) instead of silent exit 0.
3. Risk analysis flags identity-changed env vars (e.g., `DATABASE_URL` → `REDIS_URL`) as a change even when the count is unchanged.
4. All existing tests continue to pass without modification.

## API Contracts

### Ledger start
```
changeguard ledger start <entity> --category <CATEGORY> [--message "..."]

Categories: architecture, feature, bugfix, refactor, infra, security, docs, chore, tooling
```

### Verify dry-run
```
changeguard verify --dry-run
→ Verification Plan
  Runner: nextest
  • cargo nextest run --lib --bins --workspace (default: run all tests)
  • cargo clippy (from rules)
```

### Risk analysis
No CLI changes. Internal `RuntimeUsageDelta` comparison becomes identity-aware.

## Key Files

- `src/cli.rs` — `LedgerStartArgs::category` type
- `src/commands/ledger.rs` — start handler
- `src/commands/verify.rs` — dry-run plan output
- `src/output/human.rs` — `print_verify_plan` for dry-run
- `src/impact/packet.rs` — `RuntimeUsageDelta`
- `src/impact/orchestrator.rs` — risk scoring with env-var diffing

## Definition of Done

- `ledger start --category` uses enum (tab-complete, invalid value rejected by clap).
- `verify --dry-run` prints the verification plan.
- Risk analysis flags `DATABASE_URL→REDIS_URL` as a change (cardinality-1 but differs).
- All existing tests pass.
- Integration test for `verify --dry-run` plan output.
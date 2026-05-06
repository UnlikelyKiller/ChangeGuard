# Track 42: Ledger CLI Schema Consistency

## Overview
Align CLI argument patterns across all `ledger` subcommands to eliminate user confusion. Currently some subcommands use positional arguments for `entity` or `tx_id`, while others use flags (`--entity`, `--tx-id`, `--entity-pattern`). This inconsistency causes "unexpected argument" errors and forces users to check `--help` repeatedly.

## Objectives
- Audit every variant in `LedgerCommands` (`src/cli.rs`).
- Define a single schema rule and apply it uniformly.
- Improve error discoverability by ensuring `--help` accurately reflects the unified schema.

## Schema Rule (Target State)
- **Mandatory primary subject** (`entity`, `tx_id`) → positional argument.
- **All other mandatory fields** (`summary`, `reason`, `message`, `note`) → required named flags.
- **Optional identifiers / filters** → named flag (`--entity`, `--tx-id`, `--entity-pattern`).
- **All secondary metadata** (`category`, `change_type`, `issue`) → named flag.
- **Special case**: `ledger note` currently takes `entity` and `note` as two positional args. Standardize to `entity` positional + `--message` required flag, matching the single-positional pattern of `start` and `atomic`. Provide a deprecation grace period (accept old positional with a warning for 2 releases).
- **Consistency**: `Commit`, `Rollback`, and `Atomic` must follow the same pattern: primary subject positional, all other mandatory fields as required named flags.

## Success Criteria
- Every `ledger` subcommand follows the schema rule above.
- No command has more than one mandatory positional argument.
- `--help` text for every subcommand clearly indicates positional vs flag usage.
- Snapshot tests for `--help` output of every `ledger` subcommand exist and pass.
- A deprecation grace period is implemented for `ledger note` (accept old positional `note` with a warning).
- All existing CLI integration tests pass after adjustment.
- New integration tests verify CLI parsing for every `ledger` variant, including error cases (extra positionals, missing required flags).

## Architecture
- `src/cli.rs` — Modify `LedgerCommands` enum definitions; add `LedgerGlobalOpts` flattened struct for shared flags.
- `src/commands/ledger*.rs` — Update handler signatures to match new CLI shapes.
- `tests/ledger_cli_parsing.rs` — Add parsing regression tests, including `--help` snapshot tests and error-case tests.

## Testing Strategy
- **Red commit**: Write CLI parsing tests that assert the target schema. Some will fail against current definitions.
- **Green commit**: Refactor `LedgerCommands` and handlers. Ensure all tests pass.

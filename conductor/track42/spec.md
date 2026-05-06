# Track 42: Ledger CLI Schema Consistency

## Overview
Align CLI argument patterns across all `ledger` subcommands to eliminate user confusion. Currently some subcommands use positional arguments for `entity` or `tx_id`, while others use flags (`--entity`, `--tx-id`, `--entity-pattern`). This inconsistency causes "unexpected argument" errors and forces users to check `--help` repeatedly.

## Objectives
- Audit every variant in `LedgerCommands` (`src/cli.rs`).
- Define a single schema rule and apply it uniformly.
- Improve error discoverability by ensuring `--help` accurately reflects the unified schema.

## Schema Rule (Target State)
- **Mandatory primary identifiers** (`entity`, `tx_id`) → positional argument.
- **Optional identifiers / filters** → named flag (`--entity`, `--tx-id`, `--entity-pattern`).
- **All secondary metadata** (`summary`, `reason`, `message`, `note`, `category`, `change_type`, `issue`) → named flag.
- **Special case**: `ledger note` currently takes `entity` and `note` as two positional args. Standardize to `entity` positional + `--message` flag (or stdin), matching the single-positional pattern of `start` and `atomic`.

## Success Criteria
- Every `ledger` subcommand follows the schema rule above.
- No command has more than one mandatory positional argument.
- `--help` text for every subcommand clearly indicates positional vs flag usage.
- All existing CLI integration tests pass after adjustment.
- New integration tests verify CLI parsing for every `ledger` variant.

## Architecture
- `src/cli.rs` — Modify `LedgerCommands` enum definitions.
- `src/commands/ledger*.rs` — Update handler signatures to match new CLI shapes.
- `tests/cli.rs` or new `tests/ledger_cli.rs` — Add parsing regression tests.

## Testing Strategy
- **Red commit**: Write CLI parsing tests that assert the target schema. Some will fail against current definitions.
- **Green commit**: Refactor `LedgerCommands` and handlers. Ensure all tests pass.

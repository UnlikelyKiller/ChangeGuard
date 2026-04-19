# Specification: Track 13 - Final Integration and Reset Command

## Overview
Finalize the CLI by implementing the `reset` command and ensuring all subcommands are fully integrated and functional.

## Components

### Reset Command (`src/commands/reset.rs`)
- `pub fn execute_reset(force: bool) -> Result<()>`
- Deletes the `.changeguard/` directory and its contents.
- Requires `--force` or interactive confirmation (for now, just `--force`).
- Re-initializes the `.gitignore` if it was modified (optional, maybe just leave it).

### CLI Wiring
- Ensure `Ask`, `Reset`, and `Watch` (if any stubs remain) are correctly wired to their implementations.
- Add descriptive `long_about` and examples to CLI help.

### End-to-End Flow
- Implement a macro-integration test that simulates a full developer workflow:
    1. `init`
    2. `scan` (clean)
    3. Modify files
    4. `scan` (dirty)
    5. `impact`
    6. `verify`
    7. `ask`
    8. `reset`

## Verification
- Comprehensive integration tests in `tests/e2e_flow.rs`.
- Manual verification of `--help` output.

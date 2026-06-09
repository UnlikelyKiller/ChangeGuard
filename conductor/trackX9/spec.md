# Track X9: Empty-State Hints for `observability coverage`, `deploy impact`, and `tests`

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

Three commands show empty tables with no guidance when they have no data: `observability coverage`, `deploy impact --changed`, and `changeguard tests <file>`. Consistent empty-state hints (matching the pattern added for endpoints/services-diff in the I5 audit) reduce user confusion.

## Problem Statement

When these commands return no rows they print an empty table or nothing at all, leaving users unsure whether the command failed, the data is missing, or indexing is needed:

- `observability coverage` — no coverage rows → silent empty table
- `deploy impact --changed` — no staged changes affect deployment → silent empty table
- `changeguard tests <file>` — no test mappings → silent empty output

## Acceptance Criteria

1. `observability coverage` with no data prints:
   ```
   No observability coverage data found.
   Run 'changeguard index --analyze-graph' to populate.
   ```
2. `deploy impact --changed` with no impacted deployments prints:
   ```
   No deployment impact detected for current changes.
   ```
3. `changeguard tests <file>` with no mappings prints:
   ```
   No test mappings found for '<file>'.
   Run 'changeguard index' to populate test mappings.
   ```
4. All hints are styled: message in yellow, command suggestion in cyan bold.
5. `--json` mode: emit `[]` (already the case) — no hint is printed in JSON mode.

## Key Files

- `src/commands/observability.rs` — `execute_observability_coverage`
- `src/commands/deploy.rs` — `execute_deploy_impact`
- `src/commands/test_mapping.rs` — `execute_test_mapping`

## Definition of Done

- Each command shows its hint when empty.
- JSON mode is unaffected.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

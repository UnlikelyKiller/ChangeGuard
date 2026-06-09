# Track Y1: Integration Test Coverage for Untested Command Surfaces

**Status:** Planned  
**Milestone:** Y — CLI Reliability & UX Hardening  
**Priority:** Critical

## Objective

Eleven command surfaces have zero integration tests at the CLI dispatch layer. Any refactor touching their data sources, output format, or flag handling can silently regress. Add one smoke test per surface that validates the full pipeline from CLI arguments through to output — crash-free and parseable JSON where applicable.

## Problem Statement

The following commands wire into `execute_*` functions via `clap` dispatch but no integration test exercises the complete `CLI args → command handler → stdout/stderr` path:

- `changeguard config {verify, view, schema, diff}`
- `changeguard endpoints`
- `changeguard data-models`
- `changeguard observability {coverage, diff}`
- `changeguard security {boundaries, impact}`
- `changeguard services diff`
- `changeguard dead-code`
- `changeguard viz`
- `changeguard update {--migrate, --binary, --dry-run}`
- `changeguard federate {export, scan, status}`
- `changeguard audit`

These will silently break on any refactor. Adding a smoke test per surface dramatically increases confidence.

## Acceptance Criteria

1. Each of the 11 surfaces gets ≥1 integration test that:
   - Initializes a temp repo or uses an existing fixture.
   - Invokes the command via `ChangeGuard::run()` or the `execute_*` function directly with mock state.
   - Validates non-crash exit (exit 0 or documented error).
   - Validates output is parseable JSON where `--json` is supported.
2. Tests use `tempfile::tempdir()` for isolated SQLite state.
3. Tests run under `cargo nextest run --test integration` with `--test-threads=1` where they share state.
4. All existing tests continue to pass.

## API Contracts

No new CLI flags or commands. All tests are additive.

## Key Files

- `tests/integration/cli_config.rs` — config surfaces (new)
- `tests/integration/cli_surfaces.rs` — endpoints, data-models, observability, security, services (new)
- `tests/integration/cli_dead_code.rs` — dead-code (new)
- `tests/integration/cli_viz.rs` — viz (new)
- `tests/integration/cli_update.rs` — update (new)
- `tests/integration/cli_federate.rs` — federate (new)
- `tests/integration/cli_audit.rs` — audit (new)

## Definition of Done

- 11+ new integration tests covering all 11 untested surfaces.
- `cargo nextest run --test integration` passes (211 + 11 = 222+ total).
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `cargo fmt --all -- --check` passes.
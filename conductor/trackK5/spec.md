# Track K5: Operational Transparency (Config View & Audit Pagination)

## Status
Planned

## Milestone
K: Service Discovery & Storage Hardening

## Problem
1. **Configuration Blindness**: Users cannot see the "final" resolved configuration (merged from `config.toml`, `.env`, and env vars), making it hard to debug model or threshold settings.
2. **Audit Fatigue**: `changeguard ledger audit` prints the entire history at once. In large repositories, this is unmanageable and slow.

## Solution
1. **`config view` Command**:
    - Implement a new subcommand that prints the resolved `Config` struct.
    - Support `--json` for machine-readable output.
    - Annotate sources (e.g., `[env]`, `[file]`) for critical fields like API keys or base URLs.
2. **`ledger audit` Pagination**:
    - Add `--limit <N>` and `--offset <N>` flags to the `ledger audit` command.
    - Update `LedgerDb` to support `LIMIT` and `OFFSET` in the SQL query.
    - Default to a sensible limit (e.g., 50) if the terminal is interactive.

## Definition of Done (DoD)
- [ ] `changeguard config view` displays the current effective configuration.
- [ ] `changeguard ledger audit --limit 5` returns exactly the 5 most recent transactions.
- [ ] Integration test: verify pagination logic with 10+ transactions.
- [ ] CI gate passes.

# ChangeGuard Internals & Safety

This document describes the internal behaviors, safety mechanisms, and technical constraints of ChangeGuard. Use this for debugging unexpected behavior or understanding system invariants.

## Safety Mechanisms

- **Unique Transaction Index**: A `PENDING` index in the ledger prevents duplicate open transactions for the same entity.
- **Conditional Updates**: Database status updates are conditional to prevent race conditions during concurrent access.
- **Durable State Protection**: The `reset` command preserves `ledger.db` by default to prevent accidental loss of provenance history.
- **Lexical Path Normalization**: Entity paths are auto-normalized to forward-slashes relative to repo root using lexical cleaning (no filesystem canonicalization needed).
- **Process Boundaries**: Commit validators run with timeouts and isolated stdout/stderr to prevent hanging the CLI.
- **Glob Matching**: Validator and category glob patterns use `globset` for proper path matching rather than simple substring checks.
- **Atomic Rollbacks**: The `atomic_change` command rolls back the started transaction if the commit fails, preventing orphaned `PENDING` entries.
- **Federation Confinement**: Import/export logic uses secure path normalization to ensure sibling data cannot leak into unauthorized directories.
- **Viz Server Process Management**: The server writes a PID file to `.changeguard/state/viz-server.pid` on startup and cleans it up on exit. `viz-server --stop` reads this file to terminate the process safely.
- **Dead Code Scoring**: Blends three local signals (CozoDB reachability, git activity recency, test coverage) with configurable weights. Zero-weight advisory-only risk provider — does not affect risk level calculation.
- **Document Generation Resilience**: Individual template failures are logged via `tracing::warn` and skipped; the export continues with remaining templates. Empty KG produces a warning, not a failure.

## CLI Behavior Details

> [!NOTE]
> All configuration values below are defined in `.changeguard/config.toml`.

- **Auto-Reconcile Defaults**: `ledger commit` defaults to the value of `config.ledger.auto_reconcile` (which itself defaults to `true`).
- **Drift Categorization**: The watcher assigns categories to drift based on matched glob patterns in the database or config. Unmatched paths default to `FEATURE`.
- **Stale Thresholds**: Transactions older than `config.ledger.stale_threshold_hours` (default: 24h) are marked as `STALE` in `ledger status`.
- **Lexical Cleaning**: All paths are treated as UTF-8 and normalized to a canonical forward-slash format to ensure cross-platform consistency in the ledger.

# Track L7-1: Production Polish Specification

## Objective
Polish the ChangeGuard Ledger implementation for production readiness, focusing on enhanced UI, actionable errors, documentation, and CLI help, as defined in Phase L7 of the incorporation plan.

## Deliverables & Requirements

### 1. Enhanced UI
- **Color-Coded Icons**: Integrate specific icons for statuses, categories, and flags (e.g., `󰀦` for breaking, `󰄬` for committed, `󰔟` for pending, `󰀦` for unaudited/drift).
- **Consistent Table Formatting**: Unify the aesthetic across `ledger status`, `ledger search`, and `ledger audit` (padding, colors, headers, alignment).
- **Relative Timestamps**: Show relative timestamps (e.g., "2 hours ago") alongside absolute timestamps in display outputs.
- **`ledger status --compact`**: Ensure the compact flag outputs a clean, counts-only summary.

### 2. Actionable Errors
- **`LedgerError` Diagnostics**: Refine all variants of `LedgerError` (powered by `thiserror` and `miette::Diagnostic`) to include actionable `#[help(...)]` hints.
- **Example Hints**:
  - Locked tech stack rule: "Use --force to override locked rules."
  - Ambiguous transaction UUID: "Provide a longer prefix of the transaction UUID."
  - Invalid state transitions: "You must use `ledger adopt` before committing an UNAUDITED transaction."
  - Stale transaction: "Use `ledger rollback --tx-id <id>` or `ledger adopt` to clean up stale transactions."

### 3. Documentation
- **`README.md`**: Add a new section detailing the Ledger workflows, commands, and integration with ChangeGuard features.
- **`.agents/skills/changeguard/skill.md`**: Update with complete ledger command documentation, options, and contextual usage examples for AI agents.

### 4. CLI Help
- **Clap Annotations**: Add detailed `#[arg(help = "...")]` and `#[command(about = "...", long_about = "...")]` comments to all ledger subcommands in `src/commands/ledger*.rs`.
- **Examples in Help**: Embed practical command examples in the CLI `long_about` documentation for common workflows (e.g., `atomic`, `note`, `impact --ledger-start`).

## Guidelines
- **TDD**: Write unit tests for the relative-timestamp formatting and icon mapping utilities before integrating them.
- **Consistency**: Centralize output formatting logic in `src/output/` or `src/ledger/ui.rs` if necessary to ensure all commands share the same aesthetic.
- **UX**: Retain a deterministic, parseable output where applicable, but optimize human-readable terminal output. Errors should be `miette`-powered and visually clear.
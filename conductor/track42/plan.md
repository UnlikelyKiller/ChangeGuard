# Implementation Plan - Track 42: Ledger CLI Schema Consistency

## Goal
Standardize CLI argument patterns across all `ledger` subcommands so users can predict positional vs flag usage without trial-and-error.

## Proposed Changes

### 1. Audit Current Patterns [src/cli.rs]
Map every `LedgerCommands` variant:
| Command | Positional | Flags |
|---|---|---|
| `start` | `entity` | `--category`, `--message`, `--issue` |
| `commit` | `tx_id` | `--summary` (req), `--reason` (req), `--change-type`, `--breaking`, `--auto-reconcile`, `--no-auto-reconcile`, `--force` |
| `rollback` | `tx_id` | `--reason` (req) |
| `reconcile` | (none) | `--tx-id`, `--entity-pattern`, `--all`, `--reason` (req) |
| `adopt` | (none) | `--tx-id`, `--entity-pattern`, `--all`, `--reason` |
| `atomic` | `entity` | `--summary` (req), `--reason` (req), `--category` |
| `note` | `entity`, `note` | (none) |
| `status` | (none) | `--entity`, `--compact`, `--exit-code` |
| `resume` | `tx_id` (opt) | (none) |
| `register` | (none) | `--rule-type`, `--payload`, `--force` |
| `stack` | (none) | `--category` |
| `audit` | (none) | `--entity`, `--include-unaudited` |
| `adr` | (none) | `--output-dir`, `--days` |
| `search` | `query` | `--category`, `--days`, `--breaking`, `--limit` |

### 2. Apply Standardization
- `note`: Change second positional `note` to `--message` flag (required). Keep `entity` positional.
  - **Deprecation path**: For 2 releases, accept the old positional `note` and emit a warning: `"Note as a positional argument is deprecated. Use --message <note> instead."`
- `commit` / `rollback` / `atomic`: Ensure `summary` and `reason` are required named flags (already the case in clap derive, but verify `--help` text is clear).
- `reconcile` / `adopt`: Standardize `reason` — make it required on `Adopt` (currently `Option<String>`) to match `Reconcile` for audit provenance.
- `status`: `--entity` remains optional flag (consistent with optional primary identifier).
- `resume`: `tx_id` remains optional positional (consistent with optional primary).
- `search`: `query` remains positional (it is the mandatory primary subject).
- All others are already compliant or only need doc-string updates.
- Add a top-level doc comment on `LedgerCommands` documenting the schema rule for future contributors.

### 3. Update Handler Signatures
- `src/commands/ledger.rs`: `execute_ledger_note(entity, note)` → `execute_ledger_note(entity, message)`.
- Any downstream string formatting that assumed positional note input.

### 4. Add Integration Tests
- Create `tests/ledger_cli_parsing.rs`.
- For each subcommand, instantiate the CLI parser with the target invocation pattern and assert successful parse.
- Include edge cases: missing required flags, extra positional args rejected.
- Add `--help` snapshot tests for every `LedgerCommands` variant to prevent doc-string drift.
- Add a `LedgerGlobalOpts` flattened struct for shared flags (e.g., `--dry-run`) to ensure future subcommands inherit common modifiers consistently.

## Verification Plan

### Automated Tests
- `cargo test --test ledger_cli_parsing`
- `cargo test --workspace` to ensure no regressions in existing CLI tests.

### Manual Verification
- Run `changeguard ledger --help` and every sub-help to confirm text alignment.

## Definition of Done (DoD)
- [ ] **Schema Rule Documented**: A one-sentence rule is added as a doc comment on `LedgerCommands`.
- [ ] **Note Refactored**: `ledger note` uses `entity` positional + `--message` flag.
- [ ] **Deprecation Grace Period**: Old positional `note` is accepted with a warning for 2 releases.
- [ ] **Adopt Reason Standardized**: `Adopt.reason` is now required to match `Reconcile`.
- [ ] **Help Accuracy**: Every subcommand `--help` clearly shows positional vs flag usage.
- [ ] **Snapshot Tests**: `--help` output snapshot tests exist for every `ledger` variant.
- [ ] **Parsing Tests**: New integration tests cover every `ledger` variant, including error cases.
- [ ] **Zero Regression**: All existing tests pass.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.

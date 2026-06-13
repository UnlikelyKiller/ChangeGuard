# Track K3: CLI UX Polish Plan

## Phase 1: Top-Level Aliases
- [x] Refactor `execute_ledger_status` into a shared utility.
- [x] Add `Status` subcommand to `Cli` top-level enum in `src/cli.rs`.
- [x] Add `upgrade` as an alias for `update` in `src/cli.rs`.

## Phase 2: Proactive Self-Correction
- [x] Add `miette` labels and help text to `StateError::SchemaMismatch`.
- [x] Update `execute_search`, `execute_ask`, and `execute_index` to catch schema errors.
- [x] Implement `interactively_offer_migration()` utility:
    - [x] Check if `stdin` is a terminal.
    - [x] Prompt user: "Schema mismatch. Run migration? [Y/n]".
    - [x] If confirmed, execute `update --migrate`.

## Phase 3: Search Router & Mode Signaling
- [x] Implement `is_regex_likely(query)` heuristic: detect characters `^, $, ., *, +, ?, [, ], (, ), |`.
- [x] Update `execute_search`:
    - [x] If no mode flag, run `is_regex_likely`.
    - [x] Display header: `[Search: Regex]` or `[Search: Semantic]`.
- [x] Implement `--hybrid` blend: run both engines and deduplicate by `(path, line_number)`.

## Phase 4: Verification
- [x] Manual check: `changeguard status` works.
- [x] Manual check: `changeguard search "execute_.*"` (regex auto-detection).
- [x] CI Gate.

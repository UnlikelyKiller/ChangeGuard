# Track K3: CLI UX Polish Plan

## Phase 1: Top-Level Aliases
- [ ] Refactor `execute_ledger_status` into a shared utility.
- [ ] Add `Status` subcommand to `Cli` top-level enum in `src/cli.rs`.
- [ ] Add `upgrade` as an alias for `update` in `src/cli.rs`.

## Phase 2: Proactive Self-Correction
- [ ] Add `miette` labels and help text to `StateError::SchemaMismatch`.
- [ ] Update `execute_search`, `execute_ask`, and `execute_index` to catch schema errors.
- [ ] Implement `interactively_offer_migration()` utility:
    - [ ] Check if `stdin` is a terminal.
    - [ ] Prompt user: "Schema mismatch. Run migration? [Y/n]".
    - [ ] If confirmed, execute `update --migrate`.

## Phase 3: Search Router & Mode Signaling
- [ ] Implement `is_regex_likely(query)` heuristic: detect characters `^, $, ., *, +, ?, [, ], (, ), |`.
- [ ] Update `execute_search`:
    - [ ] If no mode flag, run `is_regex_likely`.
    - [ ] Display header: `[Search: Regex]` or `[Search: Semantic]`.
- [ ] Implement `--hybrid` blend: run both engines and deduplicate by `(path, line_number)`.

## Phase 4: Verification
- [ ] Manual check: `changeguard status` works.
- [ ] Manual check: `changeguard search "execute_.*"` (regex auto-detection).
- [ ] CI Gate.

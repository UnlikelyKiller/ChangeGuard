# Track GF4: Ledger Database Query Domain Split

## Objective

Decompose `src/ledger/db.rs` by query domain while preserving `LedgerDb` as the stable public facade. The user-supplied analysis reports 1185 lines and 51 methods covering transactions, drift, search, ADRs, enforcement, federation, provenance, validators, and graph linkage, with no cohesive internal query boundaries.

## Evidence

- User analysis ranks `src/ledger/db.rs` as refactor need 8/10 due to a single data-access object accumulating 10+ query domains.
- Ledger correctness is central to provenance and hooks, so this track must avoid broad call-site churn.
- `changeguard ledger status --compact` before planning reported `0 pending, 0 unaudited drift`.

## Scope

Required domain boundaries:

- `transactions`: start, commit, rollback, atomic, pending lookup, state transitions.
- `drift`: unaudited drift records, reconcile/adopt transitions, status counts.
- `search`: FTS and ledger search result mapping.
- `adr`: ADR metadata, listing, export queries, decision links.
- `enforcement`: tech stack, validators, category filters, process policy.
- `federation`: import/export and sibling-origin queries.
- `provenance`: token/file/entity provenance and graph-link data.
- `maintenance`: garbage collection, migrations helpers owned by DB layer.
- Keep `LedgerDb` methods or trait extension methods available so command code does not migrate all at once.

## Non-Goals

- Do not change SQLite schema except for test-only fixtures if needed.
- Do not replace SQLite or add a repository abstraction with only one implementation.
- Do not alter hook semantics.
- Do not directly edit `.changeguard` state.

## Implementation Notes

- Prefer private domain modules called by `LedgerDb` facade methods. The local precedent for this shape is `src/index/orchestrator.rs`, which already pairs a facade struct with free functions taking `&Connection` (`insert_file_row`, `upsert_file_row`, etc.) — follow that pattern rather than inventing a trait.
- Use `tempfile::tempdir()` for all DB tests.
- Tests should assert lifecycle invariants and foreign-key behavior.
- Keep transaction boundaries explicit around multi-statement updates.

## Verification Strategy

Targeted:

- `cargo test ledger::db`
- `cargo test ledger`
- Integration tests for `ledger start`, `ledger commit`, `ledger status`, `ledger adopt`, `ledger search`, `ledger adr list`, `ledger validator`, and `ledger graph`.

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/ledger/db.rs` is a facade over query-domain modules.
- Existing `LedgerDb` call sites keep compiling.
- Query-domain tests cover lifecycle, drift, search, ADR, validator, federation, and graph-link behavior.
- Ledger hooks and pending/drift status remain correct.
- Final verification and reinstall pass.

## Risks

- Transaction boundary changes can create partial ledger writes.
- Hook behavior may depend on subtle status messages or counts.
- Search and ADR queries can regress if FTS aliases change.

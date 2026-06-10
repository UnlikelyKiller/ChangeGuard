You are a senior Rust reviewer performing a read-only audit of a ledger database query domain split refactor.

## Context
`src/ledger/db.rs` (1,185 lines, 51 methods) was decomposed into 8 query-domain modules under `src/ledger/db/` while preserving `LedgerDb` as the stable public facade. The facade is now ~321 lines (73% reduction).

## Previous review
Subagent review returned PASS with minor observations. The most actionable was that `drift_status_counts` was `pub` but not exposed through the facade and only used in tests. This was fixed by making it module-private (`fn` instead of `pub fn`) while keeping `#[allow(dead_code)]` for clippy.

## Files to review
Please review the following files (read-only) and report any findings.

- `src/ledger/db.rs` (facade)
- `src/ledger/db/transactions.rs`
- `src/ledger/db/enforcement.rs`
- `src/ledger/db/maintenance.rs`
- `src/ledger/db/adr.rs`
- `src/ledger/db/drift.rs`
- `src/ledger/db/provenance.rs`
- `src/ledger/db/search.rs`
- `src/ledger/db/federation.rs`

## What to look for
1. **Facade completeness**: Does `LedgerDb` expose all public methods with the same signatures?
2. **Transaction boundary safety**: Are multi-statement updates wrapped in explicit transactions?
3. **Hook behavior preservation**: Does `ledger status --compact --exit-code` still work?
4. **Query correctness**: Are SQL queries moved verbatim?
5. **Test coverage**: Are domain tests adequate?
6. **Regression risks**: Any broken imports outside the ledger module?

## Expected outcome
Return either:
- **CLEAR** — no actionable findings.
- **ACTIONABLE: <list>** — specific findings with line references.

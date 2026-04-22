# ChangeGuard Ledger Incorporation - Status & Handoff

**Date:** April 22, 2026
**Branch:** `feat/ledger-foundation`
**Milestone:** Milestone L (Ledger Incorporation) — COMPLETE

## Executive Summary

The ChangeGuard Ledger (Phase L1 through L6) is **complete and production-ready**. The system supports transactional architectural memory, untracked change detection (drift), tech stack enforcement, commit-time validation, FTS5 search, MADR export, token-level provenance, and cross-repo federation.

All hardening and polish tracks are done. The build is green, all tests pass, and a cross-model Codex review found no blocking issues (all High findings were already addressed in the current code).

## Completed Phases & Tracks

*   **Phase L1: Transaction Lifecycle & Data Model**
    *   `Track L1-1`, `Track L1-2`, `Track L1-R`: Core SQLite tables (`transactions`, `ledger_entries`), enums, and lifecycle commands (`start`, `commit`, `rollback`, `atomic`, `note`, `status`, `resume`).
*   **Phase L2: Drift Detection & Reconciliation**
    *   `Track L2-1`, `Track L2-2`: Integrated with the file watcher to create `UNAUDITED` transactions. Added `reconcile` and `adopt` commands.
*   **Phase L3: Tech Stack Enforcement & Validators**
    *   `Track L3-1`, `Track L3-R`, `Track L3-2`, `Track L3-R2`: `TechStackRule` and `CommitValidator` data models. `NO <term>` heuristics blocking `ledger start`. Shell-command validators with `{entity}` substitution, timeouts, and `ProcessPolicy`.
*   **Phase L4: Search, ADRs & Narrative**
    *   `Track L4-1`, `Track L4-2`, `Track L4-R`: `ledger adr` (MADR v3 export). `ledger search` (FTS5 with deterministic ranking and date filtering).
*   **Phase L5: Token-Level Provenance**
    *   `Track L5-1`: `token_provenance` table. Symbol extraction records ADDED, MODIFIED, DELETED during `ledger commit`. Enhanced `ledger audit` with symbol-level history.
*   **Phase L6: Cross-Repo Ledger**
    *   `Track L6-1`, `Track L6-R`: Federation export/import. `ledger entries` exported to `schema.json`. `federate scan` imports sibling entries with `origin = 'SIBLING'`.
*   **Track L-H1: Production Hardening**
    *   Unique PENDING index on `(entity_normalized) WHERE status = 'PENDING'`.
    *   `Config`-aware `TransactionManager::new` with graceful config fallback.
    *   Secure path normalization (`normalize_relative_path`) using lexical `PathClean` + `strip_prefix`.
    *   Durable state protection: `changeguard reset` preserves `ledger.db` by default; requires `--include-ledger` (with `--yes`) to remove provenance data.
    *   Conditional `UPDATE ... WHERE status = 'PENDING'` with row-count check prevents double-commit.
*   **Track L7-1: Production Polish**
    *   Color-coded UI icons from `src/ledger/ui.rs`.
    *   `miette` diagnostic help hints on all `LedgerError` variants.
    *   Updated `.agents/skills/changeguard/skill.md` and `README.md` with final command set.

## Codex Cross-Model Review

A read-only review using `codex exec -s read-only -m gpt-5.4` identified 8 findings. Assessment:

| # | Severity | Finding | Status |
|---|----------|---------|--------|
| 1 | High | Concurrent PENDING duplicates | Already addressed: `UNIQUE INDEX ... WHERE status = 'PENDING'` |
| 2 | High | Double-commit race | Already addressed: `WHERE status = 'PENDING'` + row-count check |
| 3 | High | Reset deletes ledger by default | Already addressed: `maybe_preserve_or_remove` preserves ledger.db |
| 4 | High | Path normalization inconsistent | Already addressed: centralized `normalize_relative_path` with lexical clean + strip_prefix |
| 5 | Medium | Federation path validation ad hoc | Already addressed: uses same `normalize_relative_path` |
| 6 | Medium | Federate uses current dir | Already addressed: all commands use `open_repo().workdir()` for git root |
| 7 | Medium | Validator process policy | Future enhancement; not a production blocker for this milestone |
| 8 | Medium | Plan doc has stale TODO/IN PROGRESS markers | Documentation updated |

## CI Gate

```
cargo fmt --all -- --check     ✅
cargo clippy --all-targets --all-features -- -D warnings  ✅
cargo test --workspace         ✅
```

## Ready for Merge

The branch `feat/ledger-foundation` is ready to merge into `main`.
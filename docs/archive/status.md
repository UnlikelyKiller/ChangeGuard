# ChangeGuard Ledger Incorporation - Status & Handoff

**Date:** April 23, 2026
**Branch:** `main`
**Milestone:** Milestone L (Ledger Incorporation) — COMPLETE

## Executive Summary

The ChangeGuard Ledger (Phase L1 through L7, including Hardening) is **complete and production-ready**. The system supports transactional architectural memory, untracked change detection (drift), tech stack enforcement, commit-time validation, FTS5 search, MADR export, token-level provenance, and cross-repo federation.

All hardening and polish tracks are done. The build is green, all tests pass, and critical security/stability invariants (concurrency, path confinement, state protection) are fully enforced.

## Completed Phases & Tracks

*   **Phase L1: Transaction Lifecycle & Data Model**
    *   `Track L1-1`, `Track L1-2`, `Track L1-R`: Core SQLite tables, enums, and lifecycle commands (`start`, `commit`, `rollback`, `atomic`, `note`, `status`, `resume`).
*   **Phase L2: Drift Detection & Reconciliation**
    *   `Track L2-1`, `Track L2-2`: Integrated with the file watcher to create `UNAUDITED` transactions. Added `reconcile` and `adopt` commands.
*   **Phase L3: Tech Stack Enforcement & Validators**
    *   `Track L3-1`, `Track L3-R`, `Track L3-2`, `Track L3-R2`: `TechStackRule` and `CommitValidator` data models. Shell-command validators with `{entity}` substitution, timeouts, and `ProcessPolicy`.
*   **Phase L4: Search, ADRs & Narrative**
    *   `Track L4-1`, `Track L4-2`, `Track L4-R`: `ledger adr` (MADR v3 export). `ledger search` (FTS5).
*   **Phase L5: Token-Level Provenance**
    *   `Track L5-1`: Symbol extraction records ADDED, MODIFIED, DELETED during `ledger commit`. Enhanced `ledger audit` with symbol-level history.
*   **Phase L6: Cross-Repo Ledger**
    *   `Track L6-1`, `Track L6-R`: Federation export/import. `ledger` array in `schema.json`. `origin = 'SIBLING'` attribution.
*   **Track L-H1: Production Hardening**
    *   **Concurrency:** `UNIQUE INDEX` on PENDING transactions and `expected_status` validation in bulk updates prevents race conditions.
    *   **Secure Paths:** Centralized `normalize_relative_path` utility prevents repository escapes.
    *   **State Protection:** `changeguard reset` preserves `ledger.db` by default; requires `--include-ledger` for explicit deletion.
    *   **UX Fixes:** Added top-level `changeguard audit` command for discoverability.
*   **Track L7-1: Production Polish**
    *   Color-coded UI icons, refined `miette` errors, and comprehensive README/Skill documentation.
*   **Audit 2 Remediation (Tracks 19-22)**
    *   `Track 19`: Safety-bounded `reset` command.
    *   `Track 20`: Hardened determinism and error visibility (no silent fallbacks).
    *   `Track 21`: Hardened verification runner and `ProcessPolicy` enforcement.
    *   `Track 22`: Scan diff-summary integration and symbol persistence seams.

## CI Gate

```
cargo fmt --all -- --check                                  ✅
cargo clippy --all-targets --all-features -- -D warnings    ✅
cargo test --workspace                                      ✅
```

## Maintenance Notes
- The Ledger database is located at `.changeguard/state/ledger.db`.
- WAL mode is enabled for concurrency between the CLI and the optional Daemon.
- All transactional changes to the filesystem (reconcile/adopt/commit) are atomic within a SQLite transaction.

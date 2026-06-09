---
name: coding-core
description: Use this skill when writing, modifying, or reviewing Rust code in ChangeGuard. Trigger when editing .rs files, making architectural decisions, implementing features, or discussing error handling patterns, module boundaries, or determinism.
---

# Coding Core - ChangeGuard

Load this skill when writing, modifying, or reviewing Rust code in this repository.

## Retrieval Precedence

1. **Active File / Spec**: Current code and task context.
2. **Conductor Track**: `conductor/trackN/spec.md` and `plan.md` for the current track.
3. **Ledger History**: `changeguard ledger search` for architectural history (when available).
4. **Local Rules**: `.agents/rules/*.md`.
5. **Documentation**: `docs/Plan.md`, `docs/Ledger-Incorp-plan.md`, `docs/Engineering.md`.
6. **External**: `context7` for crate docs, `exa` for web search.

Training data is stale (especially for Rust 2024/latest crates). Verify versions via `context7` or `exa`.

## Rust Patterns

- **Rust Edition**: 2024
- **Error Handling**: Typed `thiserror` enums + `miette::Diagnostic` for user-facing. `anyhow` for internal infrastructure only. Never `unwrap`/`expect` in production.
- **Async**: Not used in core ChangeGuard (CLI is synchronous). Only in optional `daemon` feature (tower-lsp + tokio).

## Determinism Contract

- Sort all emitted collections before output/persistence.
- Version the impact packet schema.
- Never suppress parse/scan failure silently — annotate partial data explicitly.
- Normalize volatile fields (timestamps) in test fixtures.
- Given the same repo state and config, verification plans must be identical.

## Module Boundaries (SRP)

- `platform/` — environment-specific normalization and detection ONLY. No business logic.
- `index/` — changed-file symbol/import extraction ONLY. No repo-wide call graphing.
- `state/` — persistence, layout, migrations ONLY. No business decisions.
- `impact/` — fact assembly, scoring, explanation ONLY. Relationships compute facts, score assigns weights, reasoning formats output.
- `ledger/` — transaction lifecycle, enforcement, search ONLY. No impact analysis (that's `impact/`).

## Anti-Overengineering (YAGNI)

- Do NOT build lock managers before a real race exists.
- Do NOT build repo-wide call graphs.
- Do NOT build generalized plugin systems.
- Do NOT force data into SQLite when flat-file state is sufficient.
- Do NOT create abstraction layers with only one implementation.

## Traceability

- Use `// @cg-tx: <tx_id>` comments to link complex logic back to ledger transactions.
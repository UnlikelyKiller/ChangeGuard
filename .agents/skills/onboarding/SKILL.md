---
name: onboarding
description: Trigger this skill when starting a new session on the ChangeGuard repo, when an agent needs orientation, or when asked "where do I start?", "what's the project state?", "how does work get done here?", or "onboard me". Also load when writing/reviewing Rust code, orchestrating tracks, or using research/CI tools. Loads once per session to establish context.
---

# ChangeGuard Onboarding

You are working on **ChangeGuard** — a local-first Rust CLI that provides change intelligence, transactional provenance, and semantic search for code repositories.

## What ChangeGuard Does

ChangeGuard turns repository changes into deterministic impact packets, risk summaries, hotspot rankings, targeted verification plans, and transactional provenance records. It is a single Rust binary that:

1. **Scans** git changes and extracts symbols/imports.
2. **Scores impact** with risk levels, temporal coupling, and hotspot detection.
3. **Knowledge Graph**: Builds a native Datalog-powered graph (CozoDB) for reachability analysis.
4. **Ledger**: Tracks every change as an atomic transaction with architectural decision (ADR) export.
5. **Intelligence**: Local embedding-based search for docs, ADRs, and semantic test prediction.
6. **Observability**: Enrichments from Prometheus, service-maps, OpenSLO, and API contract matching.

## Current State

All milestones through **W** are complete as of v0.1.3.

| Milestone | Status | Summary |
|---|---|---|
| L — Ledger Incorporation | Complete | Full transaction lifecycle, drift detection, tech stack enforcement, ADR generation, federated intelligence |
| M — Observability Expansion | Complete | Local embedding client, doc crawler, semantic retrieval, Prometheus integration, OpenAPI contract matching |
| KG — Knowledge Graph | Complete | CozoDB integration and structural extraction |
| R — Architecture Refactoring | Complete | `ImpactOrchestrator` and `EnrichmentProvider` decomposition |
| S — Global Intelligence & Precision Search | Complete | Sub-millisecond Tantivy search, SCIP ingestion, semantic snippet discovery |
| T — Predictable Verification | Complete | Predictive CI Gate analysis, failure explanations via local LLM, probabilistic reordering |
| W — Surface Hardening (W1–W13) | Complete | Endpoints, services diff, data models, config schema/diff, dependency graph, test mapping, observability diff/coverage, hotspot trends, ledger graph, validator lifecycle, security boundaries, Cedar cross-surface links |

## The Conductor / Tracks System

ChangeGuard uses a **conductor/tracks** system in `conductor/` for structured incremental delivery.

- `conductor/conductor.md` — Master registry and status of all tracks.
- `conductor/trackN/spec.md` — Objective, requirements, and API contracts.
- `conductor/trackN/plan.md` — Phased task checklist with `- [ ]` checkboxes.

Track numbering history:

- **Tracks 0–40**: Original ChangeGuard v1 + Phase 2 features (Completed)
- **Tracks L1-1 through L7-1**: Ledger Incorporation (Completed)
- **Tracks G1-G7**: Native Standalone / Knowledge Graph (Completed)
- **Tracks R1-1 through R1-4**: Architectural Refactoring (Completed)
- **Tracks S1-S3**: Global Intelligence & Precision Search (Completed)
- **Tracks T1-T2**: Predictable Verification (Completed)
- **Tracks W1-W13**: Surface Hardening (Completed)

## Architecture at a Glance

```
src/
├── main.rs              — Entry point
├── commands/            — CLI command implementations (init, scan, ledger, viz, ask, etc.)
├── ledger/              — Transaction lifecycle and provenance logic
├── impact/              — Orchestrator and Enrichment Providers (API, KG, Infra, etc.)
├── index/               — AST parsing (tree-sitter) and Knowledge Graph loading
├── state/               — Persistence (SQLite + CozoDB)
├── retrieval/           — Semantic search and RAG logic
├── ai/                  — Local model client and semantic extractor
├── observability/       — Prometheus and log-scraping logic
├── contracts/           — OpenAPI/Swagger matching
└── docs/                — Native doc chunking and indexing
```

## What to Do First (New Session)

1. Read `conductor/conductor.md` — see the current active track and next planned milestone.
2. Run `changeguard doctor` — verify toolchain health.
3. Run `changeguard ledger status` — check if there is an active transaction or unaudited drift.

## Quick Reference: Commands

```bash
# Workflow loop
changeguard scan --impact       # Pre-edit impact and risk check
changeguard ledger start        # Start provenance tracking
changeguard verify              # Run predictive verification plan
changeguard ledger commit       # Finalize with ADR note

# Intelligence
changeguard ask "..."           # Narrative analysis
changeguard viz                 # HTML Knowledge Graph export

# Surface review (W1-W13)
changeguard endpoints --changed
changeguard services diff
changeguard data-models impact --changed
changeguard config schema / config diff
changeguard observability diff / observability coverage
changeguard hotspots trend / hotspots explain
changeguard security boundaries / security impact --changed
changeguard ledger graph <tx-id>
changeguard ledger validator list / doctor
```

---

## Rust Coding Standards

### Retrieval Precedence

When researching before a code change, use this order:

1. **Active file / spec** — current code and task context.
2. **Conductor track** — `conductor/trackN/spec.md` and `plan.md`.
3. **Ledger history** — `changeguard ledger search` for architectural history.
4. **Local rules** — `.agents/rules/*.md`.
5. **Documentation** — `docs/Engineering.md`, `docs/architecture.md`.
6. **External** — `context7` for crate docs, web search for ecosystem questions.

Training data is stale for Rust 2024 and recent crates. Verify API versions via `context7` or web search.

### Language & Error Handling

- **Rust edition**: 2024
- **Error handling**: Typed `thiserror` enums + `miette::Diagnostic` for user-facing errors. `anyhow` for internal infrastructure only. Never `unwrap`/`expect` in production code.
- **Async**: Not used in core ChangeGuard (CLI is synchronous). Only in the optional `daemon` feature (tower-lsp + tokio).

### Determinism Contract

- Sort all emitted collections before output or persistence.
- Version the impact packet schema.
- Never suppress parse/scan failure silently — annotate partial data explicitly.
- Normalize volatile fields (timestamps) in test fixtures.
- Given the same repo state and config, verification plans must be identical.

### Module Boundaries (SRP)

- `platform/` — environment-specific normalization and detection only. No business logic.
- `index/` — changed-file symbol/import extraction only. No repo-wide call graphing.
- `state/` — persistence, layout, migrations only. No business decisions.
- `impact/` — fact assembly, scoring, explanation only.
- `ledger/` — transaction lifecycle, enforcement, search only. No impact analysis.

### Anti-Overengineering (YAGNI)

- Do not build lock managers before a real race exists.
- Do not build repo-wide call graphs.
- Do not build generalized plugin systems.
- Do not force data into SQLite when flat-file state is sufficient.
- Do not create abstraction layers with only one implementation.

### Traceability

Use `// @cg-tx: <tx_id>` comments to link complex logic back to ledger transactions when the connection is non-obvious.

---

## Standard Operating Procedure for Tracks

### 1. Planning Phase

1. Read `conductor/conductor.md` for the next uncompleted track.
2. Run `changeguard hotspots` to identify brittle files in the target area.
3. Run `changeguard ledger status` to detect untracked changes before starting.
4. Start a transaction: `changeguard ledger start <track-name> --category <CAT>`.
5. Write `conductor/trackN/spec.md` (objective, requirements, API contracts, testing strategy) and `plan.md` (phased checklist) if not already present.
6. Update `conductor/conductor.md` with the track entry (Status: Planning).

### 2. Implementation Phase

1. **TDD loop (non-negotiable)**:
   - **Red commit**: Write failing tests asserting desired behavior. Commit.
   - **Green commit(s)**: Write production code that makes tests pass. Commit.
2. Run `changeguard scan --impact` after implementation to confirm logic hasn't leaked across module boundaries or unintentionally raised risk on brittle files.

### 3. Verification Phase (CI Gate)

Pass before every commit:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --lib --bins --workspace
```

Also run:
- `changeguard verify` — ChangeGuard's own verification plan.
- `cargo nextest run --test integration` — integration tests (use `--test-threads=1` when tests share state).

If any gate fails, fix before committing. Never use `--no-verify` unless the user explicitly requests it.

### 4. Finalization Phase

1. Mark tasks as `- [x]` in `plan.md`. Update status in `conductor/conductor.md` to `Completed`.
2. Commit with ledger: `changeguard ledger commit <tx-id> --summary "Completed Track <NAME>" --reason "<why>"`.
3. Run `changeguard ledger status` to confirm clean baseline.

### Ledger Categories

| Category | When to use |
|---|---|
| `ARCHITECTURE` | Module boundaries, SRP, determinism contracts, new subsystems |
| `FEATURE` | New CLI commands, impact enrichment, predictive verification |
| `INFRA` | SQLite migrations, embedding pipeline, CI configuration |
| `SECURITY` | Secret redaction, path confinement, process policy |
| `REFACTOR` | Internal cleanup without behavior change |
| `BUGFIX` | Defect corrections |
| `DOCS` | Track documentation, ADRs, skill files, conductor updates |
| `CHORE` | Version bumps, lockfile updates, tooling maintenance |

---

## Tooling & Research Patterns

### ChangeGuard (Self-Hosted)

ChangeGuard is the tool being developed AND the governance layer for this repo.

| Phase | Command | Purpose |
|---|---|---|
| Session start | `changeguard doctor` | Verify toolchain health |
| Before edits | `changeguard scan --impact` | Detect drift and assess blast radius |
| After implementation | `changeguard impact` | Full impact report |
| Before commit | `changeguard verify` | Run verification plan |
| On commit | `changeguard ledger commit` | Close transaction |
| Audit | `changeguard ledger status` | Ensure clean baseline |

### GitHub CLI (`gh`)

- `gh run list` — check remote CI pipeline status after a push.
- `gh issue view <n>` — read requirements before starting work.
- `gh pr status` / `gh pr diff` — self-review before final verification.

### Codebase Search

Prefer ChangeGuard's own index before grep or file reads:

```bash
changeguard index --incremental   # Refresh index (fast)
changeguard search "symbol"       # High-precision regex/text
changeguard ask "..."             # Conceptual/semantic
changeguard ask "what calls validateToken"
changeguard ask "find all Axum route handlers"
```

For deep structural questions, use `changeguard ask --semantic`. For precise symbol navigation, use SCIP-backed LSP data via `changeguard search`.

---

## Invariants (Never Break)

- **No `unwrap`/`expect`** in production code. Use `Result` + `?`.
- **Determinism**: same repo state + same config → same output.
- **Local-first**: all features work offline with local model. Network features degrade gracefully.
- **Windows paths**: prefer `camino` for UTF-8 paths. Normalize separators at boundaries.
- **Test isolation**: all tests use `tempfile::tempdir()` for SQLite. No shared global state.
- **No secrets in commits**: never commit `.env`, credentials, or API keys.
- **No editing `.changeguard/` state files** directly unless the user explicitly requests it.

## Key Reference Documents

- `docs/Engineering.md` — Engineering principles (SRP, idiomatic Rust, determinism, error visibility)
- `docs/architecture.md` — Module boundaries and data flow
- `docs/TrackingAbility.md` — Per-surface tracking scores and remaining gaps
- `docs/Features.md` — Feature surface reference
- `.agents/skills/changeguard/SKILL.md` — Day-to-day ChangeGuard command reference
- `.agents/skills/changeguard/references/commands.md` — Full command details

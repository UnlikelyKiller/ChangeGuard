---
name: onboarding
description: Trigger this skill when starting a new session on the ChangeGuard repo, when an agent needs orientation, or when asked "where do I start?", "what's the project state?", "how does work get done here?", or "onboard me". Loads once per session to establish context.
---

# ChangeGuard Onboarding

You are working on **ChangeGuard** — a local-first Rust CLI that provides change intelligence, transactional provenance, and semantic search for code repositories. This skill establishes your current context.

## What ChangeGuard Does

ChangeGuard turns repository changes into deterministic impact packets, risk summaries, hotspot rankings, targeted verification plans, and transactional provenance records. It is a single Rust binary that:

1. **Scans** git changes and extracts symbols/imports.
2. **Scores impact** with risk levels, temporal coupling, and hotspot detection.
3. **Knowledge Graph**: Builds a native Datalog-powered graph (CozoDB) for reachability analysis.
4. **Ledger**: Tracks every change as an atomic transaction with architectural decision (ADR) export.
5. **Intelligence**: Local embedding-based search for docs, ADRs, and semantic test prediction.
6. **Observability**: Enrichments from Prometheus, service-maps, and API contract matching.

## Current State

**Milestone L (Ledger Incorporation):** **COMPLETED**. Full transaction lifecycle (start/commit/rollback/atomic), drift detection, tech stack enforcement, ADR generation, and federated intelligence are production-ready.

**Milestone M (Observability Expansion):** **COMPLETED**. Local embedding client, doc crawler, semantic retrieval, Prometheus integration, and OpenAPI contract matching are fully functional.

**Milestone KG (Knowledge Graph):** **COMPLETED**. CozoDB integration and structural extraction are complete.

**Milestone R (Architecture Refactoring):** **COMPLETED**. Decomposed monolithic logic into `ImpactOrchestrator` and `EnrichmentProvider` architecture.

**Milestone S (Global Intelligence & Precision Search):** **COMPLETED**. Sub-millisecond Tantivy search, SCIP ingestion, and semantic snippet discovery.

**Milestone T (Predictable Verification):** **COMPLETED**. Predictive CI Gate analysis, failure explanations via local LLM, and probabilistic verification reordering.

## The Conductor/Tracks System

This project uses a **conductor/tracks** system in `conductor/` for structured incremental delivery.

- `conductor/conductor.md`: Master registry and status of all tracks.
- `trackN/spec.md`: The objective and API contracts.
- `trackN/plan.md`: The phased task checklist.

## TDD Discipline (Non-Negotiable)

Every feature follows the **two-commit minimum**:
1. **Red commit**: Failing tests asserting desired behavior.
2. **Green commit**: Implementation that makes tests pass.

### The CI Gate (Pass Before Every Commit)
```bash
cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo test --workspace
```

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
├── ai/                  — Local model client and semantic extractor (G7 focus)
├── observability/       — Prometheus and log-scraping logic
├── contracts/           — OpenAPI/Swagger matching
└── docs/                — Native doc chunking and indexing
```

## What to Do First

1. **Read `conductor/conductor.md`** — see the current active track.
2. **Check `changeguard doctor`** — verify toolchain health.
3. **Run `changeguard ledger status`** — check if there is an active transaction.

## Quick Reference: Commands

```bash
# Workflow loop
changeguard scan --impact   # Pre-edit check
changeguard ledger start    # Start provenance tracking
changeguard verify          # Run predictive verification
changeguard ledger commit   # Finalize with ADR note

# Intelligence
changeguard ask "..."       # Narrative analysis
changeguard viz             # HTML Knowledge Graph export
```
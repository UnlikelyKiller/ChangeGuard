---
name: onboarding
description: Trigger this skill when starting a new session on the ChangeGuard repo, when an agent needs orientation, or when asked "where do I start?", "what's the project state?", "how does work get done here?", or "onboard me". Loads once per session to establish context.
---

# ChangeGuard Onboarding

You are working on **ChangeGuard** — a local-first Rust CLI that provides change intelligence and transactional provenance for code repositories. This skill gives you everything you need to be productive.

## What ChangeGuard Does

ChangeGuard turns repository changes into deterministic impact packets, risk summaries, hotspot rankings, targeted verification plans, and transactional provenance records. It is a single Rust binary that:

1. **Scans** git changes and extracts symbols/imports
2. **Scores impact** with risk levels, temporal coupling, hotspot detection
3. **Plans verification** — predictive or config-driven test/lint commands
4. **Tracks provenance** — transaction lifecycle with ledger entries (in progress)
5. **Federates** cross-repo impact via sibling schema.json files
6. **Integrates Gemini** for AI-assisted analysis (optional)

## Current State

**Built and working (Milestones E–J, Tracks 0–40):** All original ChangeGuard features are complete — scan, impact, verify, watch, hotspots, federate, ask (Gemini), daemon (LSP), reset. 25 integration test files, ~90+ source files, 10 DB migrations (M1–M10).

**Not started:** The entire Ledger incorporation (Phases L1–L7). See `docs/Ledger-Incorp-plan.md`. No `src/ledger/` directory, no ledger CLI commands, no `LedgerConfig`, no ledger DB tables. The `changeguard` skill documents the planned ledger interface — it is aspirational, not implemented. (Check conductor\conductor.md for precise status)

**Pending task:** #9 (Add changeguard config verify subcommand — low priority).

## The Conductor/Tracks System

This project uses a **conductor/tracks** system for structured incremental delivery. It lives in `conductor/`.

### What the Conductor Is

`conductor/conductor.md` is the master file — single source of truth for all track statuses, milestone groupings, and workflow sequence. It lists every track with status, spec/plan paths, goal, and key additions.

### What a Track Is

A **track** is the atomic unit of work — a self-contained, deliverable-bounded increment. Each track has:

- `conductor/trackN/spec.md` — **specification**: objective, requirements, context, API contracts, testing strategy
- `conductor/trackN/plan.md` — **plan**: ordered task checklist with `- [ ]` checkboxes, broken into phases

### The Workflow

```
1. Plan      → architecture-planner creates spec.md + plan.md
2. Push Plan → commit and push plan to main
3. Implement → generalist creates branch, works through plan tasks
4. Review    → rust-triage-specialist or frontend-reviewer audits
5. Iterate   → if review fails, implementer fixes
6. Merge     → if review passes, merge into main
7. Next      → update conductor.md, start next track
```

### How Tracks Map to the Roadmap

- **Tracks 0–13**: Original ChangeGuard v1 (Phases 1–16, Milestones E–H)
- **Tracks 14–29**: Phase 2 feature tracks (temporal, complexity, hotspots, prediction, LSP, federation, narrative)
- **Tracks 30–40**: Audit-driven remediation (Milestones I–J) — fixes found by cross-model review
- **Next tracks (L1–L7)**: Ledger incorporation — new milestone group, track numbering TBD

## TDD Discipline (Non-Negotiable)

Every feature follows the **two-commit minimum**:

1. **Red commit**: Write failing tests that assert the desired behavior. Commit them.
2. **Green commit(s)**: Write production code that makes the tests pass. Commit.

No task is "Verified" until behavioral correctness is proven via tests. This is enforced in `.agents/rules/core-mandates.md`.

### The CI Gate (Must Pass Before Every Commit)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

If any of these fail, the commit is blocked. Fix before proceeding.

## Architecture at a Glance

```
src/
├── main.rs              — Entry point, error handling
├── cli.rs               — Clap command definitions
├── commands/            — 11 command implementations
│   ├── init.rs          │   scan.rs       │   watch.rs
│   ├── doctor.rs        │   impact.rs     │   verify.rs
│   ├── ask.rs           │   reset.rs      │   hotspots.rs
│   └── federate.rs      │   daemon.rs
├── config/              — model, load, validate, defaults, error
├── state/               — layout, storage, migrations (M1–M10), locks, reports
├── git/                 — repo, status, diff, classify, ignore
├── impact/              — analysis, packet, hotspots, redact
├── index/               — symbols, references, storage, normalize, metrics, languages/
├── verify/              — runner, results, predict, timeouts
├── gemini/              — wrapper, modes, prompt, narrative, sanitize
├── watch/               — batch, filters, debounce, normalize
├── federated/           — mod, storage, scanner, schema, impact
├── policy/              — defaults, mode, matching, protected_paths, rules, validate, load
├── platform/            — detect, shell, paths, env, process_policy
├── daemon/              — (feature-gated) LSP server
├── output/              — table, json, diagnostics, lsp
├── exec/                — boundary for process execution
└── util/                — clock, fs, hashing, process, text
```

### Module Boundaries (SRP — Strict)

| Module | Owns | Does NOT own |
|--------|------|-------------|
| `platform/` | OS detection, path normalization | Business logic |
| `index/` | Changed-file symbol/import extraction | Repo-wide call graphs |
| `state/` | Persistence, layout, migrations | Business decisions |
| `impact/` | Fact assembly, scoring, explanation | Transaction lifecycle |
| `ledger/` *(planned)* | Transaction lifecycle, enforcement, search | Impact analysis |

### Key Patterns

- **Error handling**: `thiserror` + `miette::Diagnostic` for user-facing, `anyhow` for internal. No `unwrap`/`expect` in production.
- **Determinism contract**: Sort all emitted collections. Version schemas. Never suppress failures silently. Normalize volatile fields in tests.
- **Verification runner**: `PreparedStep { executable, args }` — argv-style, not shell strings. Shell fallback only for metacharacters.
- **State**: SQLite under `.changeguard/state/ledger.db`. WAL mode. No down-migrations.
- **Config**: TOML in `.changeguard/config.toml`. One config file, one state root.

## What to Do First

When you start a session:

1. **Read `conductor/conductor.md`** — see current milestone and track statuses
2. **Check git status** — any uncommitted work?
3. **Run `changeguard doctor`** — verify toolchain health
4. **Load the relevant skill** based on your task:
   - Writing code → `/coding-core`
   - Running ChangeGuard commands → `/changeguard`
   - Using research tools → `/tooling`
   - Cross-model review → `/codex-review`

## Starting a New Track (Ledger Phases)

For the upcoming Ledger incorporation, each phase from `docs/Ledger-Incorp-plan.md` should become a track or set of tracks:

1. **Create spec**: Write `conductor/trackN/spec.md` with objective, requirements, API contracts, dependencies, testing strategy. Reference the corresponding phase in the Ledger plan.
2. **Create plan**: Write `conductor/trackN/plan.md` with phased task checkboxes. Start with TDD: test stubs first, then implementation.
3. **Update conductor.md**: Add the track entry with status "Planning".
4. **Commit and push**: The spec/plan commit is the first commit (it's the "Plan" step).
5. **Create a branch**: Start implementation on a feature branch.
6. **Red commit first**: Write the failing tests, commit.
7. **Green commits**: Implement, run CI gate, commit.
8. **Mark plan tasks**: Check off `- [x]` as you go.
9. **Update conductor.md**: Mark track as Completed when done.

## Key Reference Documents

| Document | Purpose |
|----------|---------|
| `docs/Plan.md` | Original v1 implementation plan (1262 lines) |
| `docs/Ledger-Incorp-plan.md` | Ledger incorporation plan — Phases L1–L7 (1548 lines) |
| `docs/Engineering.md` | Engineering principles (SRP, KISS/YAGNI, determinism, error visibility) |
| `conductor/conductor.md` | Master track registry and status |
| `.agents/rules/core-mandates.md` | Non-negotiable mandates (TDD, CI, security) |
| `.agents/rules/shell.md` | Windows PowerShell safety rules |
| `.agents/skills/coding-core/skill.md` | Rust patterns, module boundaries, YAGNI |
| `.agents/skills/changeguard/skill.md` | ChangeGuard command guide (includes planned ledger commands) |
| `.agents/skills/tooling/skill.md` | Sourcebot, GitHub CLI, ChangeGuard self-use patterns |
| `.agents/skills/codex-review/skill.md` | Cross-model review with Codex (GPT) |

## What Not To Do

- Do not add features beyond what the track spec requires (YAGNI)
- Do not build lock managers, repo-wide call graphs, or plugin systems
- Do not force data into SQLite when flat-file state suffices
- Do not create abstraction layers with only one implementation
- Do not commit secrets, `.env`, or API keys
- Do not use `&&` in shell commands on Windows (use `;`)
- Do not skip the CI gate (fmt + clippy + test) before committing
- Do not skip the TDD red commit — tests first, always
- Do not edit generated state under `.changeguard/` unless the user explicitly asks
- Do not treat the `changeguard` skill's ledger commands as implemented — they are planned

## Quick Reference: Commands

```bash
# Health check
changeguard doctor

# Pre-flight (before edits)
changeguard scan --impact

# Quick triage
changeguard impact --summary

# Post-flight (after edits)
changeguard verify

# CI gate (before every commit)
cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo test --workspace
```
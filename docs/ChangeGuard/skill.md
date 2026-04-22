---
name: changeguard
description: Use ChangeGuard for local-first change intelligence before, during, and after code edits. Trigger this skill whenever a repository contains ChangeGuard, the user asks about impact analysis, blast radius, risk, verification planning, hotspots, temporal coupling, Gemini-assisted review, or wants an AI agent to make safer changes with evidence from `changeguard scan`, `impact`, `verify`, or `ask`.
---

# ChangeGuard

Use this skill to make code changes with ChangeGuard's local risk, impact, and verification signals.

This file is intentionally portable:

- For Claude Code skills, copy it to a skill folder as `SKILL.md`.
- For Gemini CLI agent skills, copy it to an extension skill folder such as `skills/changeguard/SKILL.md`.
- For plain agent instructions, paste the full body into the agent's repo instructions.

## Purpose

ChangeGuard is a local-first CLI that turns repository changes into deterministic impact packets, risk summaries, hotspot rankings, targeted verification plans, and bounded Gemini context.

Use it as a safety and planning layer. It is not the source of truth for code correctness; it tells you what changed, what may be affected, and what should be verified.

## When To Use

Use ChangeGuard when:

- Starting work in a repo that already has `.changeguard/`.
- Planning a non-trivial code change.
- Reviewing staged or unstaged changes.
- Deciding which tests or checks to run.
- Estimating blast radius before editing shared code.
- Investigating risky files, hotspots, temporal coupling, or cross-repo dependencies.
- Preparing structured context for an AI coding assistant.
- Producing a handoff summary after implementation.

## First Checks

From the repository root, inspect whether ChangeGuard is available:

```bash
changeguard doctor
```

If the command is unavailable, do not invent ChangeGuard output. Tell the user it is not installed or not on `PATH`, then continue with normal repository inspection.

If installation is allowed, install ChangeGuard like a normal CLI:

```bash
curl -fsSL https://raw.githubusercontent.com/UnlikelyKiller/ChangeGuard/main/install/install.sh | sh
```

On Windows PowerShell:

```powershell
iwr https://raw.githubusercontent.com/UnlikelyKiller/ChangeGuard/main/install/install.ps1 -UseB | iex
```

After installing, open a new terminal if needed and re-run:

```bash
changeguard doctor
```

If the repo has not been initialized and the user wants ChangeGuard used here:

```bash
changeguard init
changeguard doctor
```

## Core Workflow

Before making a meaningful edit:

```bash
changeguard scan --impact
```

The `--impact` flag runs scan followed by impact analysis in one command. For separate control:

```bash
changeguard scan
changeguard impact
```

For quick triage without full output:

```bash
changeguard impact --summary
```

Read the generated report at:

```text
.changeguard/reports/latest-impact.json
```

Use the report to identify:

- `riskLevel`
- `riskReasons`
- changed files
- public symbols and imports
- runtime usage such as environment variables or config keys
- temporal couplings
- hotspots
- federated/cross-repo impact if present

After making edits:

```bash
changeguard scan --impact
changeguard verify
```

Read:

```text
.changeguard/reports/latest-verify.json
```

Use `verify` results as the primary ChangeGuard evidence for whether the planned validation passed.

## Persistent Verification Plans

ChangeGuard supports project-specific verification plans in `.changeguard/config.toml`:

```toml
[verify]
default_timeout_secs = 300

[[verify.steps]]
description = "Run project tests"
command = "cargo test -j 1 -- --test-threads=1"
timeout_secs = 300

[[verify.steps]]
description = "Check formatting"
command = "cargo fmt --check"
```

When `changeguard verify` runs without `-c`, it follows this priority:

1. **`-c` flag**: Single manual command (highest priority)
2. **Config steps**: Steps defined in `[verify]` config section
3. **Predictive mode**: Impact packet + rules + predictor
4. **Hardcoded default**: `cargo test -j 1 -- --test-threads=1`

Steps that omit `timeout_secs` inherit `default_timeout_secs`. Invalid steps (empty commands, zero timeouts) are warned and skipped rather than failing the entire config load.

## Command Guide

Use these commands by default:

```bash
changeguard scan --impact
changeguard verify
changeguard hotspots
changeguard federate status
```

Use targeted variants when appropriate:

```bash
changeguard impact --all-parents
changeguard impact --summary
changeguard verify --no-predict
changeguard verify -c "cargo clippy -- -D warnings"
changeguard hotspots --limit 20 --commits 500
changeguard hotspots --json
changeguard federate export
changeguard federate scan
changeguard reset --all --yes
```

Use Gemini-assisted reporting only when Gemini is configured and the user wants AI synthesis:

```bash
changeguard ask "What should I verify next?"
changeguard ask --mode suggest "What checks should I run?"
changeguard ask --mode review-patch "Review the current diff."
changeguard ask --narrative
```

The LSP daemon is available when built with the `daemon` feature:

```bash
changeguard daemon
```

## Strategic Reasoning for AI Agents

When acting as a coding agent, use ChangeGuard signals to adjust your strategy:

1. **Temporal Coupling (The "Hidden" Link)**: If `latest-impact.json` shows a high affinity (e.g., >70%) between a changed file and an unchanged file, you **MUST** read the unchanged file. Assume there is a logical dependency that imports alone do not show. Coupling scores now use recency weighting — recent shared commits count more than old ones. Files appearing in fewer than 5 commits or pairs sharing fewer than 3 commits are filtered out to reduce scaffolding noise.
2. **Hotspots (The "Danger Zone")**: Files with high hotspot scores are "brittle." If you must edit a hotspot, prioritize refactoring or extremely high test coverage. Avoid adding complexity to an already complex hotspot.
3. **Federated Impact (Cross-Repo)**: If `federated_impact` warnings appear, your change might break a sibling repository. You must explain this risk to the user and suggest an `export-schema` to verify the contract.
4. **Predictive Verification**: If `verify` suggests tests that seem unrelated to your change, **trust the predictor**. It is likely based on historical failure correlations that aren't obvious from the code alone. If you have a `[verify]` config section, those steps run before predictive mode.
5. **Stale Data**: If you see a `data_stale` warning or a `data-stale` diagnostic, run `changeguard scan` and `changeguard impact` immediately to refresh the local cache.

## How To Interpret Results

Treat `riskLevel` as a routing signal:

- `Low`: small or isolated change. Run ChangeGuard's suggested verification and any obvious local tests.
- `Medium`: inspect affected files, imports, risk reasons, and predicted verification targets before choosing tests.
- `High`: slow down. Inspect temporal couplings, hotspots, public API changes, protected paths, runtime/config usage, and cross-repo links before finalizing.

Treat `prediction_warnings` in `latest-verify.json` as important. If prediction inputs degraded, explain that the verification plan may be incomplete.

Treat hotspot and temporal coupling findings as test-selection evidence, not proof of a bug.

The `impact --summary` flag outputs a single-line triage: `RISK risk | N changed | N couplings | N partial`. Use it for quick go/no-go decisions before reading the full report.

## Editing Rules

Before edits:

1. Run or inspect `changeguard scan --impact` when feasible.
2. Use `latest-impact.json` to understand blast radius.
3. Prefer small, scoped edits when ChangeGuard reports high risk, hotspots, or broad couplings.

During edits:

1. Do not edit generated state under `.changeguard/` unless the user explicitly asks.
2. Do not commit `.env`, local secrets, SQLite state, report artifacts, or transient ChangeGuard files.
3. Respect the repo's existing tests, config, and rules before adding new verification commands.

After edits:

1. Run `changeguard impact` again.
2. Run `changeguard verify`.
3. Run any additional tests required by the repo or by the changed files.
4. Summarize the ChangeGuard evidence in the final response.

## Final Response Template

When reporting work that used ChangeGuard, include:

```text
ChangeGuard:
- impact: <low|medium|high>, with key risk reasons
- affected areas: <important files/modules/symbols>
- hotspots/couplings: <notable findings or "none material">
- verification: <commands run and pass/fail result>
- warnings: <prediction/degradation warnings or "none">
```

Keep the summary factual. If ChangeGuard could not run, say why and name the fallback verification you performed.

## Safety Notes

ChangeGuard is local-first, but its `ask` command invokes Gemini CLI. Before using `changeguard ask`, confirm the user is comfortable sending sanitized, truncated repository context to Gemini.

Never paste secrets from `.env`, config files, reports, or terminal output into prompts or final responses. If ChangeGuard reports redaction or prompt truncation, mention that it occurred without revealing the redacted value.

## Repo-Specific ChangeGuard Notes

- Required verification commands (run in order, all must pass):
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --workspace -- -D warnings`
  3. `cargo test --workspace`
- **Always use** `changeguard verify -c "cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace" --timeout 300` — bare `changeguard verify` only runs tests by default.
- Protected paths: `crates/lexbase-core/src/lib.rs`, `crates/lexbase-db/migrations/`
- High-risk modules: `crates/lexbase-ingest/src/engine.rs`, `crates/lexbase-retrieval/src/orchestrator.rs`
- Known slow tests: Postgres integration test (ignored by default, requires running Postgres with pgvector)
- Cross-repo dependencies: none
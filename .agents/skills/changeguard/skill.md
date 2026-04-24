---
name: changeguard
description: Use ChangeGuard for local-first change intelligence and transactional provenance before, during, and after code edits. Trigger this skill whenever a repository contains ChangeGuard, the user asks about impact analysis, blast radius, risk, verification planning, hotspots, temporal coupling, Gemini-assisted review, architectural transactions, drift detection, or wants an AI agent to make safer changes with evidence from `changeguard scan`, `impact`, `verify`, `ledger`, or `ask`.
---

# ChangeGuard

Use this skill to make code changes with ChangeGuard's local risk, impact, verification, and provenance signals.

This file is intentionally portable:

- For Claude Code skills, copy it to a skill folder as `SKILL.md`.
- For Gemini CLI agent skills, copy it to an extension skill folder such as `skills/changeguard/SKILL.md`.
- For plain agent instructions, paste the full body into the agent's repo instructions.

## Purpose

ChangeGuard is a local-first CLI that turns repository changes into deterministic impact packets, risk summaries, hotspot rankings, targeted verification plans, bounded Gemini context, and transactional provenance records.

Use it as a safety and planning layer. It is not the source of truth for code correctness; it tells you what changed, what may be affected, what should be verified, and records why the change was made.

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
- Recording why a change was made (transactional provenance).
- Detecting untracked architectural drift.

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

## Ledger Workflow (Transactional Provenance)

For tracked changes, wrap edits in a ledger transaction:

**Before edits:**

```bash
changeguard ledger start --entity src/main.rs --category FEATURE --message "Add auth module"
```

**After edits and verification:**

```bash
changeguard verify
changeguard ledger commit --tx-id <id> --change-type MODIFY --summary "Added auth module" --reason "API endpoints need authentication" --verification-status verified --verification-basis tests
```

**For single-file surgical edits** (start + commit in one call, with `--auto-reconcile` to resolve drift):

```bash
changeguard ledger atomic --entity src/config.rs --category REFACTOR --summary "Extract config validation" --reason "SRP: separate loading from validation"
```

**For lightweight changes** (e.g., documentation):

```bash
changeguard ledger note --entity docs/api.md "Update endpoint docs"
```

**To find a pending transaction** (instead of hunting for UUIDs):

```bash
changeguard ledger resume
```

**To reconcile drift** detected by the watcher:

```bash
changeguard ledger reconcile --all --reason "Intentional local changes"
```

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

### Impact & Scan

```bash
changeguard scan --impact           # Before edits: full change intelligence
changeguard impact --all-parents    # Include side-branch commits in coupling analysis
changeguard impact --summary         # One-line triage: RISK | N changed | N couplings
```

### Verification

```bash
changeguard verify                         # Run configured or predicted verification
changeguard verify -c "cargo clippy -- -D warnings"   # Manual single command
changeguard verify --no-predict            # Skip predictive suggestions
```

### Reset

```bash
changeguard reset                          # Preserves config, rules, and ledger.db
changeguard reset --remove-config          # Remove .changeguard/config.toml
changeguard reset --remove-rules           # Remove .changeguard/rules.toml
changeguard reset --include-ledger --yes   # Destructive: wipe ledger.db
changeguard reset --all --yes              # Destructive: wipe the entire .changeguard tree
```

### Audit & Search

```bash
changeguard audit [--entity PATH] [--include-unaudited]  # Holistic provenance view
changeguard ledger audit [--entity PATH]                 # Same as above (legacy alias)
changeguard ledger search QUERY [--category CAT] [--days N] [--breaking] [--limit N] # FTS5 search
```

### Ledger (Provenance)

```bash
changeguard ledger start --entity PATH --category CAT [--message TEXT] [--issue REF]
changeguard ledger commit --tx-id ID --summary TEXT --reason TEXT [--change-type TYPE] [--breaking] [--auto-reconcile | --no-auto-reconcile]
changeguard ledger rollback --tx-id ID --reason TEXT
changeguard ledger atomic --entity PATH --summary TEXT --reason TEXT [--category CAT]
changeguard ledger note --entity PATH NOTE
changeguard ledger resume [ID]                              # Find most recent PENDING tx or resume specific    
changeguard ledger status [--entity PATH] [--compact]       # Holistic view or entity history
changeguard ledger reconcile [--tx-id ID] [--entity-pattern GLOB] [--all] --reason TEXT
changeguard ledger adopt [--tx-id ID] [--entity-pattern GLOB] [--all] [--reason TEXT]
changeguard ledger stack [--category CAT]                   # Show tech stack and validators
changeguard ledger register --rule-type TYPE --payload JSON [--force]   # Add enforcement rules
changeguard ledger adr [--output-dir DIR] [--days N]        # Export decisions to MADR
```
### Hotspots & Federation

```bash
changeguard hotspots --limit 20 --commits 500
changeguard hotspots --json
changeguard federate export
changeguard federate scan
changeguard federate status
```

### Gemini-Assisted Reporting

Use only when Gemini is configured and the user wants AI synthesis:

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

## Categories

| Category | Covers |
|---|---|
| `ARCHITECTURE` | High-level system design, multi-module contracts |
| `FEATURE` | New user-facing or internal functionality |
| `BUGFIX` | Defect repairs |
| `REFACTOR` | Structural improvement without behavior change |
| `INFRA` | CI, git hooks, Docker, build system |
| `TOOLING` | Internal scripts, dev tooling |
| `DOCS` | Documentation, README, ADRs |
| `CHORE` | Dependencies, formatting, minor cleanup |

## Strategic Reasoning for AI Agents

When acting as a coding agent, use ChangeGuard signals to adjust your strategy:

1. **Temporal Coupling (The "Hidden" Link)**: If `latest-impact.json` shows a high affinity (>70%) between a changed file and an unchanged file, you **MUST** read the unchanged file. Assume there is a logical dependency that imports alone do not show.
2. **Hotspots (The "Danger Zone")**: Files with high hotspot scores are "brittle." If you must edit a hotspot, prioritize refactoring or extremely high test coverage. Avoid adding complexity to an already complex hotspot.
3. **Federated Impact (Cross-Repo)**: If `federated_impact` warnings appear, your change might break a sibling repository. You must explain this risk to the user.
4. **Predictive Verification**: If `verify` suggests tests that seem unrelated to your change, **trust the predictor**. It is likely based on historical failure correlations.
5. **Drift Detection**: If `changeguard ledger status` shows UNAUDITED entries, files were modified without a pending transaction. Either reconcile them (`ledger reconcile`) or adopt them (`ledger adopt`) before proceeding. This is the trust-and-verify model in action.
6. **Tech Stack Enforcement**: `ledger start` will reject a transaction if the `planned_action` or entity path violates registered tech stack rules (e.g., using a forbidden library). Category-to-stack mappings with glob filters scope rules to specific entity paths.
7. **Commit Validation**: `ledger commit` runs shell-command validators (e.g., linters, tests). Validators with glob patterns only run when the entity matches. `ERROR` level failures block the commit.
8. **Data Classification**: `changeguard reset` preserves `ledger.db` by default. Use `--include-ledger` (requires `--yes`) to remove provenance data alongside derived state.
9. **Auto-Reconcile from Config**: `ledger commit` defaults to `config.ledger.auto_reconcile` (default: true). Use `--no-auto-reconcile` to override. Drift is reconciled for the entity being committed.
10. **Drift Categorization**: Watcher-detected drift assigns category from DB or config watcher patterns (matched by glob). Unmatched paths default to `FEATURE`.
11. **Configurable Stale Threshold**: PENDING transactions older than `config.ledger.stale_threshold_hours` (default: 24h) show a stale indicator in `ledger status`.

## How To Interpret Results

Treat `riskLevel` as a routing signal:

- `Low`: small or isolated change. Run suggested verification and any obvious local tests.
- `Medium`: inspect affected files, imports, risk reasons, and predicted verification targets before choosing tests.
- `High`: slow down. Inspect temporal couplings, hotspots, public API changes, protected paths, runtime/config usage, and cross-repo links before finalizing.

The `impact --summary` flag outputs a single-line triage: `RISK risk | N changed | N couplings | N partial`. Use it for quick go/no-go decisions.

## Editing Rules

Before edits:

1. Run or inspect `changeguard scan --impact` when feasible.
2. Use `latest-impact.json` to understand blast radius.
3. For tracked changes, run `changeguard ledger start`.
4. Prefer small, scoped edits when ChangeGuard reports high risk, hotspots, or broad couplings.

During edits:

1. Do not edit generated state under `.changeguard/` unless the user explicitly asks.
2. Do not commit `.env`, local secrets, SQLite state, report artifacts, or transient ChangeGuard files.
3. Respect the repo's existing tests, config, and rules before adding new verification commands.

After edits:

1. Run `changeguard impact` again.
2. Run `changeguard verify`.
3. Run any additional tests required by the repo or by the changed files.
4. For tracked changes, run `changeguard ledger commit`.
5. Summarize the ChangeGuard evidence in the final response.

## Final Response Template

When reporting work that used ChangeGuard, include:

```text
ChangeGuard:
- impact: <low|medium|high>, with key risk reasons
- affected areas: <important files/modules/symbols>
- hotspots/couplings: <notable findings or "none material">
- verification: <commands run and pass/fail result>
- ledger: <tx_id if tracked, or "untracked">
- warnings: <prediction/degradation/drift warnings or "none">
```

## Safety Notes

- Full UUID is always displayed in `ledger start`.
- `ledger resume` finds the most recent PENDING transaction.
- Entity paths are auto-normalized to forward-slashes relative to repo root using lexical path cleaning (no filesystem canonicalization).
- Watcher-detected drift creates UNAUDITED transactions; category is determined by watcher pattern matching.
- Commit validators run with timeouts to prevent hanging the CLI.
- Validator glob patterns use proper glob matching (globset), not substring checks.
- `atomic_change` rolls back the started transaction if commit fails, preventing orphaned PENDING entries.
- Federation import uses the same secure path normalization as local ledger entries.

## Repo-Specific Notes

- Required verification commands (run in order, all must pass):
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo test --workspace`
- Protected paths: `src/config/model.rs`, `src/state/migrations.rs`, `src/impact/packet.rs`
- High-risk modules: `src/impact/temporal.rs`, `src/verify/runner.rs`, `src/ledger/transaction.rs`
- Cross-repo dependencies: none

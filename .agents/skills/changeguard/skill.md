---
name: changeguard
description: Use ChangeGuard for local-first change intelligence and transactional provenance. Trigger this skill whenever a repository contains `.changeguard/`, or the user mentions "risk," "impact analysis," "blast radius," "hotspots," "temporal coupling," "drift," "verification planning," "architectural transactions," or wants safer changes with evidence from `scan`, `impact`, `verify`, or `ledger`.
---

# ChangeGuard

Use this skill to perform risk analysis, impact assessment, and record transactional provenance for code changes. ChangeGuard provides a safety and planning layer to understand what changed, what is affected, and what must be verified.

## When NOT to use

Avoid triggering ChangeGuard for:
- **Trivial Formatting**: Pure whitespace changes or `cargo fmt` runs.
- **Dependency Bumps**: Simple version updates in lockfiles with no API changes.
- **Explicit Bypass**: When the user explicitly says "just make the edit" or "bypass ChangeGuard."
- **Non-Code Assets**: Edits to binary files, media, or temporary scratch files.

## Availability & Fallback

Check if ChangeGuard is initialized in the current repository:

```bash
changeguard doctor
```

If the command is missing, see [install.md](./references/install.md). If you cannot install it, continue with standard repository tools but inform the user that ChangeGuard signals are unavailable.

## Core Workflow

Before making a meaningful edit, assess the risk:

```bash
changeguard scan --impact
```

Read the generated report at `.changeguard/reports/latest-impact.json` to identify risk level, affected symbols, temporal couplings, and runtime dependencies (environment variables, config keys).

After making edits, verify the change:

```bash
changeguard verify
```

Evidence of successful validation is stored in `.changeguard/reports/latest-verify.json`. For full command details, see [commands.md](./references/commands.md).

## Ledger Workflow (Provenance)

For tracked changes, record the intent and outcome in the ledger.

**Tracked Edit (Manual):**
1. `changeguard ledger start --entity <path> --category <CAT> --message "Intent"`
2. *Perform edits...*
3. `changeguard ledger commit --tx-id <id> --summary "Done" --reason "Why"`

**Surgical Edit (Atomic):**
Use this for single-file changes where the start and commit happen together:
```bash
changeguard ledger atomic --entity <path> --category <CAT> --summary "Task" --reason "Goal"
```

**Lightweight Note:**
Use this to add metadata to a file without a formal transaction:
```bash
changeguard ledger note --entity <path> "Metadata note"
```

## Strategic Reasoning

Adjust your coding strategy based on ChangeGuard signals:

1. **Temporal Coupling**: If a changed file has a high affinity (>70%) with an unchanged file, you **MUST** read that unchanged file. Logical dependencies often exist where imports do not.
2. **Hotspots**: Files with high hotspot scores are brittle. Prioritize refactoring or higher test coverage when editing them. **Note**: When entering an unfamiliar codebase, `changeguard hotspots` serves as an orientation map of where complexity is concentrated.
3. **Federated Impact**: If `federated_impact` warnings appear, your change may break a sibling repository. Explain this risk to the user.
4. **Predictive Verification**: Trust the `verify` command's suggestions, even if they seem unrelated; they are often based on historical failure correlations.
5. **Drift Detection**: If `ledger status` shows `UNAUDITED` entries, files were modified outside a transaction. Use `ledger reconcile` or `ledger adopt` before continuing.

## Interpreting Results

Use the `riskLevel` from impact reports to route your effort:
- **Low**: Small/isolated change. Run suggested verification.
- **Medium**: Inspect affected symbols and risk reasons before choosing tests.
- **High**: Slow down. Inspect temporal couplings, public API changes, and cross-repo links before finalizing.

For quick triage, use `changeguard impact --summary`.

## Editing Rules

**Before Edits:**
- Run `changeguard scan --impact`.
- For tracked changes, run `changeguard ledger start`.

**During Edits:**
- Do not edit state under `.changeguard/`.
- Do not commit transient ChangeGuard files or SQLite state.

**After Edits:**
- Run `changeguard verify` and any repo-specific tests.
- For tracked changes, run `changeguard ledger commit`.

## Final Response Template (Optional)

For substantive changes, summarize the evidence:
```text
ChangeGuard:
- impact: <low|medium|high> (risk reasons)
- hotspots/couplings: <findings or "none">
- verification: <commands run and result>
- ledger: <tx_id or "untracked">
```

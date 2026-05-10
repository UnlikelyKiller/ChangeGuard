---
name: codex-review
description: Use this skill when you want a cross-model code review, a second opinion on changes, or an independent audit before committing. Trigger when the user asks for a review, a second pair of eyes, cross-model review, Codex review, or wants GPT/Codex to examine code. Also trigger before final verification on high-risk changes.
---

# Codex Cross-Model Review

Different AI models catch different issues. Use Codex (GPT-based) as an independent read-only reviewer to supplement Claude-based development. This is especially valuable before committing high-risk changes, after substantial refactors, or when the ChangeGuard impact report shows elevated risk.

## When To Use

- Before committing high-risk changes (ARCHITECTURE, FEATURE, BUGFIX categories)
- After a substantial refactor spanning multiple files
- When ChangeGuard reports `riskLevel: High` or broad temporal couplings
- After implementing a full phase from the Ledger incorporation plan
- When you want a second opinion on design decisions
- Before creating a PR

## Quick Review (One-Shot)

Run a non-interactive read-only review:

```powershell
codex exec -C "C:\dev\ChangeGuard" -s read-only -m gpt-5.4 -o review.md "Review the current phase of work. Compare the current git diff against the base branch, identify bugs, regressions, missing tests, risky patterns, and unclear assumptions. Do not modify files. Give findings ordered by severity (critical/high/medium/low), then list the most important follow-up checks."
```

Key flags:

| Flag | Purpose |
|------|---------|
| `-C <path>` | Set workspace root |
| `-s read-only` | Prevent the reviewer from modifying files |
| `-m gpt-5.4` | Use GPT-5.4 for the review (different training than Claude) |
| `-o review.md` | Write final review text to file |
| `--json` | Machine-readable output (for CI integration) |

## Targeted Review

Review specific files or a specific diff:

```powershell
codex exec -C "C:\dev\ChangeGuard" -s read-only -m gpt-5.4 -o review.md "Review ONLY these files: src/ledger/transaction.rs src/ledger/db.rs. Check for: unsafe patterns, missing error handling, inconsistent status transitions, and SQL injection risks. Do not modify files."
```

Review a specific commit range:

```powershell
codex exec -C "C:\dev\ChangeGuard" -s read-only -m gpt-5.4 -o review.md "Review the changes between HEAD~5 and HEAD. Focus on: does the transaction lifecycle handle all edge cases? Are there any paths where a PENDING transaction could become orphaned? Do not modify files."
```

## ChangeGuard-Aware Review

Include ChangeGuard signals in the review prompt so Codex can prioritize its findings:

```powershell
codex exec -C "C:\dev\ChangeGuard" -s read-only -m gpt-5.4 -o review.md "Run 'changeguard impact --summary' to see the current risk level. Then review the git diff with that risk context. Focus on: (1) files with high hotspot scores, (2) temporally coupled files that weren't changed but might need updates, (3) protected paths. Do not modify files."
```

## Interactive Review

For deeper investigation where you want back-and-forth:

```powershell
codex -C "C:\dev\ChangeGuard" -m gpt-5.4
```

Then inside the TUI:

```
/review
```

This opens an interactive review session. Use `/model` to switch models mid-session if needed.

## Review Profiles

For frequent reviews, create a profile in `~/.codex/config.toml`:

```toml
[profiles.deep-review]
model = "gpt-5.4"
sandbox = "read-only"
ask_for_approval = "never"
```

Then invoke:

```powershell
codex exec -p deep-review -C "C:\dev\ChangeGuard" -o review.md "Review the current diff for bugs, regressions, and missing tests. Do not modify files."
```

## Reading the Output

After a one-shot review, read the output:

```bash
cat review.md
```

The review should contain findings ordered by severity. Address critical and high findings before committing. Medium and low findings can be tracked as follow-up.

## Integration with ChangeGuard Workflow

1. Run `changeguard scan --impact` — get risk signals
2. Make your changes
3. Run `changeguard impact` — see blast radius
4. Run `codex exec -s read-only ...` — get cross-model review
5. Address critical/high findings
6. Run `changeguard verify` — run configured verification
7. Commit with `changeguard ledger commit`

## Safety Notes

- Always use `-s read-only` for reviews. The reviewer should never modify files.
- Do not pass secrets, API keys, or `.env` contents in review prompts.
- Codex output is written by a different model — its suggestions may not align with this project's conventions (Rust 2024, miette errors, determinism contract). Evaluate suggestions against the coding-core skill before applying.
- Review output is advisory, not authoritative. You still make the final call.

## Cost Awareness

Each `codex exec` call consumes API tokens. For routine low-risk changes, skip the cross-model review. Reserve it for:

- High-risk or high-complexity changes
- Phase completion reviews (L1, L2, etc.)
- Pre-PR reviews
- When you're uncertain about a design decision
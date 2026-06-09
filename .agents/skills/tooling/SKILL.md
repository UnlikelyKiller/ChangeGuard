---
name: tooling
description: Use this skill when using Sourcebot, GitHub CLI, or ChangeGuard's own commands for research, CI verification, or codebase exploration. Trigger when the task involves searching the codebase, checking CI, reviewing PRs, or running changeguard scan/impact/verify/ledger commands.
---

# Tooling & Research - ChangeGuard

Load this skill when using research or operational tools on this codebase.

## Sourcebot (Deep Research)

Sourcebot is the primary tool for codebase navigation and architectural understanding.

### Patterns

1. **Symbol Search**: Use `search_code` with `useRegex: true` to find trait implementations (e.g., `impl.*HistoryProvider`).
2. **Context Mapping**: Before modifying a module, use `ask_codebase` to identify hidden dependencies or non-obvious side effects.
3. **Commit Analysis**: Use `list_commits` to understand the intent behind recent changes when `changeguard ledger search` is insufficient.

## GitHub CLI (`gh`)

The `gh` CLI is the bridge between local development and the repository.

### Patterns

1. **CI Verification**: Use `gh run list` to check remote CI pipeline status after a push.
2. **Issue Integration**: Use `gh issue view` to read requirements before starting work.
3. **PR Management**: Use `gh pr status` and `gh pr diff` to self-review before final verification.

## ChangeGuard (Self-Hosted)

ChangeGuard is the tool being developed AND the governance tool for this repo.

### Patterns

1. **Pre-Flight**: `changeguard scan --impact` before edits. `changeguard ledger start --entity <path> --category <cat> --description <text>` for tracked changes.
2. **Post-Flight**: `changeguard verify` and `changeguard ledger commit --tx-id <id> --change-type MODIFY --summary <text> --reason <text>` after edits.
3. **Quick Triage**: `changeguard impact --summary` for a one-line risk assessment.
4. **Artifact Suggestions**: `changeguard ledger artifacts` to see what git diff says changed.

## Key Reference Documents

- `docs/Plan.md` — Original implementation plan and architecture boundaries
- `docs/Ledger-Incorp-plan.md` — Plan for incorporating Project Ledger functionality
- `docs/Engineering.md` — Engineering principles review (SRP, idiomatic Rust, determinism contract, error visibility)
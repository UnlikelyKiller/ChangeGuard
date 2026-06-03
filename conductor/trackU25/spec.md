# Track U25: Skill & Help Text Accuracy

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P1 — Friction / Documentation Drift

---

## Problem Statement

Various documentation and `--help` text mismatches exist across the CLI and skills:
1. `SKILL.md` says `index --auto-index` but the flag does not exist on `index`.
2. References mention `ledger resume` and `ledger note`, which do not exist in CLI subcommands.
3. `ledger register validator --help` lists duplicate `-c` short flag for category and command.

## Acceptance Criteria

**AC1:** Correct line 79 in `SKILL.md` to reference `index --incremental` instead of `--auto-index`.
**AC2:** Remove or implement the undocumented `ledger resume` and `ledger note` from documentation references.
**AC3:** Change the short flag for command in `ledger register validator` to `-x` or similar, resolving the duplicate `-c` collision.

## Design Notes

- Edit `.agents/skills/changeguard/SKILL.md` and `references/commands.md`.
- Modify CLI clap annotations in `src/cli.rs`.

## Verification

- Run `changeguard ledger register validator --help` and verify no duplicate `-c` exists.

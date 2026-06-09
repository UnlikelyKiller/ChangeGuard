# Track Y2: Standardize JSON Output Contract

**Status:** Planned  
**Milestone:** Y — CLI Reliability & UX Hardening  
**Priority:** High

## Objective

Establish and enforce a project-wide JSON output contract so `changeguard X --json | jq` works reliably across all command surfaces. Currently `--json` goes to stdout for some commands, `--out <file>` writes to files for others, and human-readable metadata (status lines, table headers) interleaves with JSON on stdout in several commands, breaking piped consumption.

## Problem Statement

Audit of JSON output across ChangeGuard surfaces reveals inconsistent patterns:

| Command | `--json` behavior | Issue |
|---------|-------------------|-------|
| `scan --impact --json` | stdout | ✅ Clean JSON |
| `scan --impact --out <path>` | file | ✅ Clean file |
| `hotspots --json` | stdout | ✅ Clean JSON |
| `impact` (standalone) | writes `latest-impact.json` to reports dir | ❌ No `--json` flag |
| `ledger search` | text only | ❌ No `--json` flag |
| `ledger status` | text only | ❌ No `--json` flag |
| `config view --json` | stdout | ✅ Clean form |
| `config verify --json` | stdout | ✅ Clean form |
| `security boundaries --json` | stdout | ✅ Clean form |
| `security impact --json` | stdout | ✅ Clean form |
| `observability coverage --json` | stdout | ✅ Clean form |
| `endpoints --json` | stdout | ✅ Clean form |

Mixed output (status text interleaved with JSON) appears in some commands — the human-friendly text goes to stdout alongside JSON, making `| jq` consume non-JSON lines.

## Acceptance Criteria

1. All commands with `--json` output **only** valid JSON to stdout (no interleaved human text).
2. All human-readable metadata (status lines, progress messages, table headers) goes to stderr or is suppressed when `--json` is active.
3. `--out <file>` writes output to the specified file path.
4. Missing `--json` flags added to: `ledger search`, `ledger status`.
5. `changeguard X --json | jq` works without errors for every command that supports `--json`.

## API Contracts

No breaking changes to existing CLI flags. New `--json` flags for `ledger search` and `ledger status`:

```
changeguard ledger search --json "query"
changeguard ledger status --json [--compact]
```

Output rules:
- `--json`: stdout gets ONLY the JSON object/array; stderr gets progress text.
- `--out <path>`: file gets the output; stdout gets human text (or nothing if `--quiet`).
- No flag: stdout gets human-readable text (tables, status lines).

## Key Files

- `src/commands/impact.rs` — add `--json` / `--out` aliases
- `src/commands/ledger.rs` — `status` and `search` JSON output
- `src/output/human.rs` — global `--json` output stderr routing
- `src/cli.rs` — flag definitions

## Definition of Done

- Every `--json` surface outputs pure JSON to stdout with no human text.
- `ledger search --json` and `ledger status --json` added.
- `changeguard impact --json` and `changeguard impact --out <path>` added.
- `changeguard X --json | jq` works across all surfaces.
- All existing tests pass.
- Integration tests verify JSON-only output for at least 3 surfaces.
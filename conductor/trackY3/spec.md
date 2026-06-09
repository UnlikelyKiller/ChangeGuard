# Track Y3: Consolidate `scan --impact` vs Standalone `impact`

**Status:** Planned  
**Milestone:** Y — CLI Reliability & UX Hardening  
**Priority:** High

## Objective

Eliminate the confusing overlap between `changeguard scan --impact` and `changeguard impact`. Both produce impact analysis but with different flags, different output paths, and different internal code paths. Users should have one clear way to get impact analysis, with consistent flags.

## Problem Statement

Currently:
- `changeguard scan --impact` runs impact analysis as part of a scan, supports `--json`, `--out`, `--summary`
- `changeguard impact` runs impact analysis standalone, writes `latest-impact.json` to `.changeguard/reports/`, has **no** `--json` or `--out` flags
- `changeguard impact` (standalone) calls `execute_impact_silent()` which skips human output — different code path from `scan --impact`
- `scan --impact --summary` calls `execute_impact` with `summary=true` — yet another path

This is confusing: "do I use `scan --impact` or `impact`?" The answer depends on whether you need JSON, a file, or a summary, which is not discoverable from help text.

## Acceptance Criteria

1. `changeguard impact --json` outputs JSON to stdout (matching `scan --impact --json`).
2. `changeguard impact --out <path>` writes impact report to the specified path.
3. Internal code paths between `scan --impact` and standalone `impact` are merged or deduplicated so behavior is identical for the same flags.
4. `scan --impact --json` continues to work exactly as before (no breaking change).
5. Help text for both `scan --impact` and `impact` cross-references each other.

## API Contracts

```
changeguard impact [--json] [--out <path>] [--summary]
```

- `--json` → serialize `ImpactPacket` to stdout.
- `--out <path>` → write report to path instead of default reports directory.
- `--summary` → print brief risk line instead of full table.

## Key Files

- `src/cli.rs` — ImpactArgs flag definitions
- `src/commands/impact.rs` — execute_impact code paths
- `src/impact/orchestrator.rs` — ImpactOrchestrator API

## Definition of Done

- `changeguard impact --json` produces identical output to `changeguard scan --impact --json`.
- `changeguard impact --out test.json` writes to the specified file.
- Internal code paths merged (no duplicate logic).
- All existing test references to `scan --impact` / `impact` continue to pass.
- Integration test validates that both entry points produce identical results.
# Track K8: CLI Consistency (Scan Impact JSON)

## Status
Completed

## Milestone
K: Service Discovery & Storage Hardening

## Problem
`changeguard scan --impact` produces human-readable output by default but lacks the `--json` and `--out` flags available in other commands (like `viz` or `hotspots`), leading to friction for automation.

## Objective
Enable standardized JSON output for the `scan --impact` command.

## Success Criteria
- [x] `changeguard scan --impact --json` prints valid JSON to stdout.
- [x] `changeguard scan --impact --out report.json` saves JSON to a file.
- [x] Output format is consistent with `src/impact/packet.rs`.
- [x] CI gate passes.

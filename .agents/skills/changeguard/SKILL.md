---
name: changeguard
description: Use this skill when working in a repository initialized with `.changeguard/` and the task involves code edits, reviews, impact/risk analysis, verification planning, drift handling, ledger provenance, or deciding what tests to run. Before meaningful edits, run ChangeGuard scan/impact; after edits, run verification and report unresolved drift or ledger state.
---

# ChangeGuard

Use ChangeGuard as the local safety layer and engineering intelligence engine for code changes. It provides impact analysis, hotspot and temporal-coupling signals, verification planning, and transactional provenance.

## Core Capabilities

- **Search & Discovery**: High-performance regex (Tantivy), precise LSP navigation (SCIP), and conceptual semantic search (local embeddings).
- **Impact Analysis**: Deep "blast radius" analysis across 20+ specialized providers (Infra, Contracts, Observability, Temporal).
- **Predictable Verification**: Bayesian test reordering and CI failure prediction.
- **Transactional Ledger**: Record architectural intent and decisions as atomic transactions.
- **Documentation Generation**: Export Knowledge Graph data to Markdown/Mermaid passive documentation (`index --export-docs`).
- **Dead Code Detection**: Confidence-based dead code detection blending graph reachability, git activity, and test history (`dead-code` command).
- **Live Visualization**: WebSocket-based Arc Diagram for real-time Knowledge Graph updates (`viz-server`, `viz-server --stop`).

## Philosophy: CLI-First Intelligence

ChangeGuard is a **CLI-first** tool and **explicitly rejects MCP/Server/Cloud architecture** for v1. It provides structured, "Gemini-ready" context directly via its CLI outputs. Use ChangeGuard commands as your primary discovery and safety tools.

## Default Workflow

1. Check availability when uncertain:

   ```bash
   changeguard doctor
   ```

2. Check current provenance state:

   ```bash
   changeguard ledger status
   ```

3. Before meaningful code edits, assess impact:

   ```bash
   changeguard scan --impact
   ```

4. Read `.changeguard/reports/latest-impact.json` when it exists. Use it to
   identify risk level, hotspots, temporal couplings, affected symbols, runtime
   dependencies, and verification hints.

5. Make the smallest scoped change that satisfies the task.

6. After edits, run:

   ```bash
   changeguard verify
   ```

   Also run any repo-specific tests needed for the touched files.

7. Report the outcome: impact/risk signals used, verification run, and any
   unresolved pending transactions, drift, or unavailable ChangeGuard command.

## When To Skip

Skip ChangeGuard only for trivial formatting, simple dependency lockfile updates,
binary/media changes, temporary scratch files, or when the user explicitly says
to bypass it.

## If Commands Fail

- If `changeguard` is unavailable, continue with normal repo tools and tell the
  user ChangeGuard signals were unavailable.
- If `ledger status` shows unaudited drift, reconcile or adopt before continuing
  unless the user directs otherwise.
- If `scan --impact` cannot complete, continue cautiously and include the error
  in the final report.
- Do not edit `.changeguard/` state files directly.

## Ledger Provenance

For tracked manual edits:

```bash
changeguard ledger start <entity> --category <CAT> --message "Intent"
# edit files
changeguard ledger commit <tx-id> --summary "Done" --reason "Why"
```

For surgical one-command provenance:

```bash
changeguard ledger atomic <entity> --category <CAT> --summary "Task" --reason "Goal"
```

## Reasoning Rules

- If temporal coupling is above 70% for an unchanged file, inspect that file.
- If hotspots are reported, bias verification toward those files first.
- If KG reachability identifies downstream nodes, inspect them before finalizing.
- Treat hooks and CI gates as enforcement. Treat this skill as guidance.

## Working On ChangeGuard Itself

After changing ChangeGuard source code, rebuild and reinstall before relying on
the `changeguard` command:

```bash
cargo install --path .
```

## References

- Command details: `references/commands.md` (includes ledger, impact, dead-code, viz-server, doc generation, watch)
- Install fallback: `references/install.md`
- Architecture/internal notes: `references/internals.md`

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

ChangeGuard is a **native standalone binary** — invoke it as `changeguard <command>`. It has **zero external dependencies** (no Python, Node, or specific runtime required).

Check if it is available and initialized in the current repository:

```bash
changeguard doctor
```

If the command is missing, see [install.md](./references/install.md). If you cannot install it, continue with standard repository tools but inform the user that ChangeGuard signals are unavailable.

## Core Workflow

Before making a meaningful edit, assess the risk:

```bash
changeguard scan --impact
```

Read the generated report at `.changeguard/reports/latest-impact.json` to identify risk level, affected symbols, temporal couplings, runtime dependencies (environment variables, config keys), and — when configured — relevant documentation decisions, production observability signals, and affected API contracts.

After making edits, verify the change:

```bash
changeguard verify
```

Evidence of successful validation is stored in `.changeguard/reports/latest-verify.json`. For full command details, see [commands.md](./references/commands.md).

## Indexing (Observability & Intelligence)

ChangeGuard uses a native indexing pipeline to build its intelligence:

```bash
changeguard index --docs           # index markdown/text docs (crawl, chunk, embed)
changeguard index --contracts      # index OpenAPI 3.x / Swagger 2.0 specs
changeguard index --analyze-graph  # refresh structural graph and compute centrality
```

These populate the **CozoDB Knowledge Graph** and embedding store. Re-indexing skips unchanged files via content-addressed hashing.

## AI Backend (Ask Command)

The `ask` command supports two AI backends for narrative analysis:

```bash
changeguard ask --backend local "review this change"   # local LLM (llama-server)
changeguard ask --backend gemini "analyze the impact"  # Gemini API (default)
```

Auto-selection: if `prefer_local = true` and a local base URL is configured, Local is used; otherwise Gemini. Set `GEMINI_API_KEY` for Gemini.

## Configuration (Key Sections)

In `changeguard.toml`:

```toml
[local_model]
base_url = "http://localhost:8081"         # or CHANGEGUARD_LOCAL_MODEL_URL
prefer_local = true

[docs]
include = [".changeguard/docs/*.md", "README.md"]
chunk_tokens = 512

[observability]
prometheus_url = ""                        # Prometheus API base URL
log_paths = []                             # log file paths to scan

[contracts]
spec_paths = []                            # OpenAPI/Swagger spec file globs

[coverage]
enabled = true                             # Master toggle for enrichment features
```

## Impact Packet Enrichment

When configured, impact reports include these enrichment sections:

| Field | Source | Description |
|---|---|---|
| `relevant_decisions` | Knowledge Index | Semantically relevant documentation chunks and architectural context |
| `observability` | Prometheus/Logs | Real-time production signals (latency, error rate, anomalies) |
| `affected_contracts` | Contract Index | Public API endpoints potentially affected by the change |
| `trace_config_drift` | Traces Provider | Changes to OTEL, Jaeger, or Datadog collector configurations |
| `sdk_dependencies_delta` | SDK Provider | New or modified third-party SDK integrations (Stripe, AWS, etc.) |
| `service_map_delta` | Service Provider | Impact on inferred service topology and cross-service edges |
| `data_flow_matches` | Coupling Provider | Semantic coupling between API handlers and data models |
| `deploy_manifest_changes`| Deploy Provider | Changes to Dockerfiles, K8s, Terraform, or Helm charts |
| `ci_gates` | CIGate Provider | CI pipeline gates and job triggers associated with changed files |
| `hotspots` | Hotspot Provider | Historical instability and complexity scores for changed files |
| `temporal_couplings` | Coupling Provider | Files that frequently change together (temporal affinity) |
| `kg_reachability` | Knowledge Graph | Datalog-based reachability analysis for semantic side-effects |

All enrichment degrades gracefully. Risk elevation from observability/contract signals escalates `risk_level` (Low→Medium→High) without overwriting rule-based risk reasons.

## Native Knowledge Graph (CozoDB)

ChangeGuard maintains a native Datalog graph in `.changeguard/state/ledger.cozo`. 

- Use `changeguard viz` to export an interactive HTML visualization of the graph.
- Use `changeguard index --analyze-graph` to re-run community detection (Louvain) and centrality scoring.

## Root Cause & Test Prediction

When `semantic_weight > 0`, the `verify` command queries past test outcomes by diff embedding similarity and blends semantic scores with rule-based predictions:

```bash
changeguard verify --explain   # show which past outcomes influenced each prediction
```

## Ledger Workflow (Provenance)

For tracked changes, record the intent and outcome in the ledger.

**Tracked Edit (Manual):**
1. `changeguard ledger start <entity> --category <CAT> --message "Intent"`
2. *Perform edits...*
3. `changeguard ledger commit <tx-id> --summary "Done" --reason "Why"`

**Surgical Edit (Atomic):**
```bash
changeguard ledger atomic <entity> --category <CAT> --summary "Task" --reason "Goal"
```

## Strategic Reasoning

1. **Temporal Coupling**: If a changed file has a high affinity (>70%) with an unchanged file, you **MUST** read that unchanged file.
2. **Hotspots**: Files with high hotspot scores are brittle. Prioritize refactoring when editing them.
3. **KG Reachability**: If the Knowledge Graph flags downstream nodes as reachable from your change, inspect those nodes even if they aren't in your direct imports.
4. **Drift Detection**: If `ledger status` shows `UNAUDITED` entries, use `ledger reconcile` or `ledger adopt` before continuing.

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

## Working on ChangeGuard Itself

When editing ChangeGuard's own source code, rebuild and reinstall after every change:
```bash
cargo install --path .
```

# ChangeGuard — Product Overview & Commercial Viability

> A local-first change intelligence platform that provides cryptographic provenance,
> impact analysis, knowledge graph, and AI governance for software development.

---

## 1. Executive Summary

ChangeGuard is a single Rust binary (`cargo install changeguard`) that turns
repository changes into deterministic impact packets, risk summaries, hotspot
rankings, targeted verification plans, and cryptographically signed provenance
records. It works fully offline with optional LLM integration (local model or
Ollama Cloud fallback).

**What makes it unique:** No competing product combines cryptographic change
ledgers, local-first knowledge graphs, intent capture via git hooks, and
deterministic verification planning in a single zero-dependency CLI.

**Commercial thesis:** The 2025–2026 AI governance market needs tools that
prove *what changed, who approved it, what risk it carries, and whether it was
AI-generated* — without sending code to a third-party cloud. ChangeGuard fills
this gap, but currently lacks the team/enterprise surfaces needed to sell to
engineering managers.

---

## 2. Current Capabilities (as of v0.1.3)

### 2.1 Change Intelligence

| Capability | CLI Command | Details |
|---|---|---|
| Impact analysis | `scan --impact`, `impact` | Files changed, symbols, imports, risk level, risk reasons |
| Risk scoring | automatic | Per-file and overall risk (TRIVIAL / MEDIUM / HIGH) with path-weighted scoring |
| Hotspot detection | `hotspots`, `hotspots explain <path>` | Bus factor, complexity × frequency, log-normalized scores |
| Temporal coupling | automatic during impact | Co-change patterns across >75% threshold |
| Dead code detection | `dead-code` | Blends graph reachability + git activity + test coverage |
| Env var usage tracking | `config diff` | Tracks which env vars are read vs declared, identity-aware comparison |
| Runtime usage analysis | automatic | Captures env var, config key, and file/dir usage from current and previous commits |

### 2.2 Knowledge Graph (CozoDB Datalog)

| Capability | CLI Command | Details |
|---|---|---|
| Symbol indexing | `index --analyze-graph` | Tree-sitter AST parsing for Rust, TS, Python, Go, C#, Java, etc. |
| Call graph | queryable via KG | Function/method call edges between files |
| Centrality computation | `index --analyze-graph` | Entry point detection, fan-in/fan-out |
| Dependency graph | `dependencies list` | Cargo.lock → Package nodes + DependsOn edges |
| SCIP/LSIF ingestion | `index --scip` | Precise, compiler-grade symbol resolution |
| Graph visualization | `viz` | D3.js force-directed HTML export |
| Transaction→entity edges | `ledger graph <tx-id>` | Affects edges from tx to changed files |

### 2.3 Ledger & Provenance

| Capability | CLI Command | Details |
|---|---|---|
| Transaction lifecycle | `ledger start / commit / rollback / atomic` | Full PENDING → COMMITTED / ROLLED_BACK state machine |
| Cryptographic signing | automatic | Ed25519 signatures on every committed transaction |
| Signature verification | `ledger status --verify-signatures` | Validates all committed entry signatures |
| Intent capture | git `commit-msg` hook | LLM drafts intent (what/why/risk/tickets), TUI confirmation |
| Drift detection | `ledger status` | Detects untracked file changes after transaction commit |
| Reconciliation | `ledger reconcile / adopt` | Resolve drift via explicit adopt or reconcile |
| ADR generation | `ledger adr` | MADR-format architecture decision records |
| FTS5 search | `ledger search` | Full-text search across all transactions |
| SOC2 evidence export | `ledger audit --json` | Multi-section health report with commit velocity, churn, CI trends |
| Federation | `federate scan / export / status` | Cross-repo ledger sharing with schema export |

### 2.4 Semantic Intelligence

| Capability | CLI Command | Details |
|---|---|---|
| Code search (BM25) | `search` | Sub-millisecond Tantivy trigram search |
| Semantic search | `search --semantic` | HNSW vector search with local embeddings |
| AI Q&A | `ask [--backend local|gemini]` | Context-assembled LLM queries with KG fallback |
| Semantic snippet retrieval | `ask --semantic` | Tree-sitter chunked code → embedding → similar search |
| Doc indexing | `index --docs` | Documentation chunking and embedding |
| Contract matching | `index --contracts`, `contracts` | OpenAPI spec parsing, endpoint→code matching |

### 2.5 Verification

| Capability | CLI Command | Details |
|---|---|---|
| Verification planning | `verify` | Builds plan from config, rules, or auto-prediction |
| Predictive CI analysis | `verify --explain` | Historical outcome prediction with failure explanations |
| Probabilistic reordering | automatic | Orders verification steps to minimize time-to-first-failure |
| Semantic test prediction | automatic | Blends embedding similarity with rule-based scores |
| Health check | `verify --health` | Bounded (<5s) executable availability + ledger health probe |
| Dry-run | `verify --dry-run` | Compressed grouped impact display (CallGraph, Temporal, TestMapping) |
| Signature verification | `verify` (internal) | Validates ledger entry signatures against stored public keys |

### 2.6 Observability & Contracts

| Capability | CLI Command | Details |
|---|---|---|
| Prometheus query | `observability diff` | Live metric thresholds, risk elevation on breach |
| Log scanning | automatic | Local log file pattern scanning |
| OpenSLO parsing | `observability coverage` | SLOs → service/endpoint risk links |
| OpenAPI matching | `contracts` | Endpoint descriptions ↔ changed files, breaking API change detection |

### 2.7 Surface Coverage (Milestone W)

| Surface | CLI Command | Rating |
|---|---|---|
| Endpoints & auth | `endpoints --changed` | 9/10 |
| ADRs & decisions | `ledger adr list` | 9/10 |
| Service boundaries | `services diff` | 9/10 |
| Data models & migrations | `data-models impact --changed` | 9/10 |
| Config & env vars | `config view / diff / schema` | 9/10 |
| CI/CD & deploy | `deploy impact --changed` | 9/10 |
| Dependencies & advisories | `dependencies list / audit` | 9/10 |
| Test mapping | `tests <file>` | 9/10 |
| Observability & SLO | `observability coverage / diff` | 9/10 |
| Hotspot trends | `hotspots trend` | 10/10 |
| Ledger & validators | `ledger graph / validator` | 10/10 |
| Security boundaries | `security boundaries / impact` | 9/10 |

### 2.8 Platform

- **Language:** Rust 2024
- **OS:** Windows (primary), macOS/Linux via cross-compile
- **Storage:** SQLite (ledger, indexes) + CozoDB (knowledge graph)
- **LLM support:** llama-server (local), Ollama (local + cloud), Gemini API
- **No external services required:** everything works offline
- **Determinism contract:** same repo + same config = same output

---

## 3. Commercial Viability Assessment

### 3.1 Market Timing (2025–2026)

Three converging trends create a window:

1. **AI governance urgency** — Enterprises adopting AI coding tools need to
   prove which changes were AI-generated, by which model, and who reviewed them.
   Verity by ProvenanceCode exists for this but is private preview only.

2. **Local-first compliance** — Regulated industries (finance, healthcare,
   defense) cannot send source code to SaaS impact analysis tools. ChangeGuard's
   local-first architecture is a hard requirement for these buyers.

3. **MCP ecosystem** — AI coding agents (Claude Code, Cursor, Copilot) need
   structured codebase context. ChangeGuard's existing IPC bridge is a
   ready-made MCP server.

### 3.2 Competitive Landscape

| Competitor | Overlap | Differentiator |
|---|---|---|
| **RepoKit** ($15–79/mo) | PR impact, deploy risk scoring | Cloud-only, no ledger, no AI provenance |
| **Sonde** (trial) | Symbol graph, blast radius | Cloud-only, no crypto, no intent capture |
| **Verity by ProvenanceCode** | Signed records, AI governance | No impact analysis, no graph, private preview |
| **Chisel** (OSS, MIT) | Test impact, risk scoring | No ledger, no graph DB, no CLI workflow |
| **RepoGraph** (OSS, MIT) | Bus factor, blast radius | No provenance, no ledger, no verification |

### 3.3 What's Missing for Commercial Viability

The CLI engine is production-ready. The gaps are all in **distribution,
team surfaces, and packaging**:

| Priority | Gap | Why It Matters | Effort |
|---|---|---|---|
| **P0** | No web UI | Managers buy tools, not CLIs. No dashboard means no enterprise sales. | 3–6 mo |
| **P0** | No MCP server packaging | AI agent integration is the fastest adoption path. IPC bridge exists but isn't `npm install`-able. | 1–2 wk |
| **P1** | No team/multi-user | Single-developer ledger doesn't help teams coordinate. | 2–3 mo |
| **P1** | No GitHub/GitLab app | PR comments with risk scores are the standard enterprise integration pattern. | 2–4 wk |
| **P1** | macOS/Linux as primary | Windows-first excludes >85% of developer market. | 1–2 mo |
| **P2** | No SaaS tier | Enterprises overwhelmingly prefer SaaS over self-hosted CLI. | 3–6 mo |
| **P2** | No pricing/packaging | No `pricing page`, no `changelog`, no `docs site`. The project has zero web presence. | 2–4 wk |
| **P3** | No onboarding wizard | `changeguard init` works but a guided `changeguard setup` would reduce friction. | 1–2 wk |
| **P3** | No telemetry (opt-in) | Can't prove usage to investors without aggregate usage data. | 1 wk |
| **P3** | No enterprise features | SSO/SAML, audit export, role-based access, dedicated support. | 2–3 mo |

### 3.4 Recommended Pricing Model

```
┌──────────────┬──────────────────────┬──────────────────────────┐
│ Tier         │ Price                │ What's included          │
├──────────────┼──────────────────────┼──────────────────────────┤
│ Free (OSS)   │ $0                   │ CLI, local-only,         │
│              │                      │ single developer,        │
│              │                      │ all existing features    │
├──────────────┼──────────────────────┼──────────────────────────┤
│ Pro          │ $19/mo               │ MCP server, team web     │
│              │                      │ dashboard, GitHub app,   │
│              │                      │ Slack webhook, 5 users   │
├──────────────┼──────────────────────┼──────────────────────────┤
│ Enterprise   │ $99/mo per seat      │ SSO/SAML, on-prem        │
│              │ or $5k/yr flat       │ option, SLA, compliance  │
│              │                      │ export, audit log,       │
│              │                      │ dedicated support        │
└──────────────┴──────────────────────┴──────────────────────────┘
```

---

## 4. What Needs to Be Built

### 4.1 Web UI (P0 — The Product)

A `changeguard web` command that starts a local HTTP server serving a
React/TypeScript frontend. The REST API surfaces the existing CozoDB + SQLite
data.

**Screens:**

1. **Dashboard** — Change feed (last 7 days), risk summary, hotspot trend chart,
   pending transactions count, verification health status. Filters by repo, risk
   level, category, author.

2. **Impact Explorer** — Search/browse past impact packets. Visual diff of risk
   levels over time. Click a change → see symbols changed, risk reasons,
   temporal couplings, predicted test impact.

3. **Knowledge Graph** — Interactive D3.js/Cytoscape graph explorer. Filter by
   node type (files, symbols, services, endpoints, ADRs). Search. Zoom into
   transaction neighborhoods.

4. **Ledger** — Searchable, filterable transaction table. Click → see full
   provenance: what changed, why, who signed, cryptographic signature chain.
   ADR history with supersession tree.

5. **Compliance Hub** — Read-only. One-click SOC2 evidence export (ZIP).
   Signature chain validation. Commit velocity chart. Oldest unaddressed ADR.
   Hotspot delta since last audit.

6. **Verification History** — Pass/fail rate per step over time, slowest
   commands, correlation explorer (which changes caused failures).

7. **Settings** — Config file viewer (with secret redaction). LLM backend
   selection. Verification step configuration. MCP server status.

### 4.2 MCP Server (P0 — Fastest Adoption Path)

Package the existing IPC bridge as a pluggable MCP server:

```
npx @changeguard/mcp-server    # or
npm install -g @changeguard/mcp
changeguard mcp                 # runs the MCP stdio/SSE server
```

**Tools to expose:**

| Tool | Description |
|---|---|
| `scan` | Run impact scan on current repo |
| `search` | Search code (BM25 or regex) |
| `ask` | Semantic Q&A with context assembly |
| `ledger_status` | Current pending/unaudited state |
| `ledger_search` | Full-text search transactions |
| `hotspots` | Current hotspot rankings |
| `dependencies_list` | Package dependency graph |
| `endpoints_changed` | API endpoints affected by current diff |
| `security_boundaries` | Security policy graph |
| `viz_graph` | Export subgraph for AI context |

### 4.3 GitHub/GitLab App (P1)

A GitHub App that:
- Comments risk scores on PRs
- Links to transaction provenance
- Blocks merges on HIGH risk changes (configurable)
- Shows hotspot deltas

### 4.4 Team Features (P1)

- Shared ledger across developers in a repo
- PR risk summaries with assignee
- Team verification dashboard
- Notification webhooks (Slack, Teams, email)

### 4.5 Web Presence (P2)

- Landing page at `changeguard.dev`
- Documentation site (docs.changeguard.dev)
- Interactive demo / playground
- Pricing page
- Blog / changelog
- GitHub Sponsors / OSS funding

### 4.6 Enterprise (P2–P3)

- SSO/SAML (Google Workspace, Okta, Azure AD)
- Read-only audit log
- On-prem deployment via Docker + docker-compose
- Usage-based billing telemetry (opt-in, verifiable)
- SLA guarantees

---

## 5. Commercial Priorities Summary

```
Now (1 month)          │  1–3 months            │  3–6 months
───────────────────────┼────────────────────────┼────────────────────────
MCP server packaging   │  Web UI (MVP)          │  Team features
GitHub App             │  macOS/Linux builds    │  Enterprise SSO
Landing page           │  SaaS tier             │  On-prem deploy
changeguard web (MVP)  │  Pricing page          │  Telemetry
DESIGN.md brand        │  CLI onboarding wizard │  Compliance automation
```

---

## 6. DESIGN.md — ChangeGuard Web UI Brand & Design System

> This section is a `DESIGN.md`-compatible specification for Google Stitch,
> Claude Design, and AI frontend generation tools. It describes the visual
> identity for the `changeguard web` UI.

```yaml
---
name: ChangeGuard
version: alpha
description: Developer-first change intelligence dashboard. Trustworthy,
  precise, local-first. The UI feels like a developer tool — dark,
  dense, informative, with a terminal-inspired aesthetic but real
  dashboard usability.

colors:
  primary: "#00E5A0"
  primaryMuted: "#00B87A"
  surface: "#0D1117"
  surfaceAlt: "#161B22"
  surfaceRaised: "#1C2333"
  border: "#30363D"
  borderMuted: "#21262D"
  textPrimary: "#E6EDF3"
  textSecondary: "#8B949E"
  textMuted: "#6E7681"
  danger: "#F85149"
  dangerMuted: "#DA3633"
  warning: "#D29922"
  warningMuted: "#9E6A03"
  success: "#3FB950"
  successMuted: "#238636"
  info: "#58A6FF"
  infoMuted: "#1F6FEB"
  accent: "#FF7B72"
  # Risk-level semantic colors
  riskHigh: "#F85149"
  riskMedium: "#D29922"
  riskLow: "#3FB950"
  riskTrivial: "#8B949E"

typography:
  body:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: 0.875rem
    lineHeight: 1.5
  bodySmall:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: 0.75rem
    lineHeight: 1.5
  heading:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: 1.25rem
    fontWeight: 600
    lineHeight: 1.3
  headingLarge:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: 1.5rem
    fontWeight: 700
    lineHeight: 1.3
  mono:
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace"
    fontSize: 0.8125rem
    lineHeight: 1.4
  monoSmall:
    fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace"
    fontSize: 0.6875rem
    lineHeight: 1.4

rounded:
  sm: 4px
  md: 6px
  lg: 8px
  xl: 12px
  full: 9999px

spacing:
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
  xxl: 48px

components:
  Card:
    backgroundColor: "{colors.surfaceAlt}"
    rounded: "{rounded.lg}"
    padding: "{spacing.lg}"
    border: "1px solid {colors.border}"
  Button:
    rounded: "{rounded.md}"
    padding: "{spacing.sm} {spacing.md}"
    fontSize: 0.8125rem
    fontWeight: 500
  ButtonPrimary:
    backgroundColor: "{colors.primary}"
    textColor: "#000000"
    rounded: "{rounded.md}"
    padding: "{spacing.sm} {spacing.md}"
  Badge:
    rounded: "{rounded.full}"
    padding: "2px {spacing.sm}"
    fontSize: 0.6875rem
    fontWeight: 600
  Table:
    borderCollapse: collapse
    headerBackgroundColor: "{colors.surface}"
    rowHoverBackgroundColor: "{colors.surfaceRaised}"
    border: "1px solid {colors.border}"
  Sidebar:
    backgroundColor: "{colors.surface}"
    width: 260px
    borderRight: "1px solid {colors.border}"
  TopNav:
    backgroundColor: "{colors.surfaceAlt}"
    borderBottom: "1px solid {colors.border}"
    height: 48px
  RiskIndicator:
    rounded: "{rounded.full}"
    width: 8px
    height: 8px
  SearchInput:
    backgroundColor: "{colors.surface}"
    border: "1px solid {colors.borderMuted}"
    rounded: "{rounded.md}"
    padding: "{spacing.sm} {spacing.md}"
    textColor: "{colors.textPrimary}"
  Tab:
    borderBottom: "2px solid transparent"
    activeBorderColor: "{colors.primary}"
    activeTextColor: "{colors.textPrimary}"
    inactiveTextColor: "{colors.textMuted}"
    padding: "{spacing.sm} {spacing.md}"
---

## Overview

ChangeGuard is a developer tool first. The web UI should feel like
it belongs next to a terminal — dark theme by default, dense information
density, monospace for data, and a green/teal accent palette that echoes
terminal success output. It's not a marketing site; it's a power tool.

The visual language is inspired by:
- GitHub's dark theme (familiar to every developer)
- Terminal UIs (dense, keyboard-navigable, minimal chrome)
- Monitoring dashboards (Grafana, Datadog) for the trend/risk views
- The ChangeGuard CLI's own color scheme (green success, red danger,
  yellow warning, cyan info)

## Colors

The palette is dark-first with a distinctive teal/emerald primary
(`#00E5A0`) that stands out against the deep navy surfaces. Semantic
risk colors map directly to the CLI's risk levels (HIGH → red,
MEDIUM → yellow, LOW → green, TRIVIAL → gray).

- **Primary (#00E5A0):** Interactive elements, active states, links,
  loading indicators. Evokes terminal success output.
- **Surface (#0D1117):** Main background. Exact GitHub dark default.
- **SurfaceAlt (#161B22):** Card backgrounds, sidebar, code blocks.
- **SurfaceRaised (#1C2333):** Hover states, dropdowns, modals.
- **Border (#30363D):** Default border color for containers.
- **TextPrimary (#E6EDF3):** Default body text.
- **Danger (#F85149):** HIGH risk, errors, destructive actions.
- **Warning (#D29922):** MEDIUM risk, stale data, warnings.
- **Success (#3FB950):** LOW risk, healthy state, passed checks.
- **Info (#58A6FF):** Information, links, neutral alerts.
- **Accent (#FF7B72):** Hotspot highlights, attention-grabbing elements.

## Typography

Two font families: Inter for UI (clean, legible at small sizes) and
JetBrains Mono for data (paths, transaction IDs, symbols, commands).
Density is high — body text at 14px, monospace at 13px, small labels at 11px.

## Layout & Spacing

The layout follows a standard developer-tool three-column hierarchy:
1. **Sidebar (260px):** Navigation, project selector, status indicators
2. **Main content:** The active view
3. **Detail panel (optional):** Contextual information on selection

Content density follows the 8px grid. Cards and sections use 16px
padding internally, 24px between sections. The layout should work at
1280px wide minimum, scaling up to fill wider monitors.

## Elevation & Depth

- **Cards** sit flush on surfaceAlt with a 1px border — no shadows.
- **Dropdowns, modals, popovers** use `box-shadow: 0 8px 24px rgba(0,0,0,0.4)`
  to lift above the surface.
- **Hover states** raise slightly via background color change
  (surfaceAlt → surfaceRaised), not elevation.

## Components

### Cards
Used for summary metrics (risk level, pending transactions, verification
health), change feed items, and configuration sections. No title bar —
the content itself is the heading.

### Risk Indicators
Small 8px circles colored by semantic risk level. Used inline next to
file paths, transaction entries, and change feed items.

### Tables
Dense, borderless except for column headers and row dividers. Each row
is 36px tall. Hover highlights the row. Sortable columns where applicable
(date, risk level, category).

### Badges
Used for categories (FEATURE, BUGFIX, etc.), risk labels, and status
indicators (PENDING, COMMITTED, UNAUDITED). Rounded pill shape, colored
by semantic meaning. The CLI's existing badge colors should carry over:
- Architecture → purple
- Feature → green
- Bugfix → red
- Refactor → blue
- Infra → cyan
- Security → orange
- Docs → gray
- Chore → muted

### Navigation (Sidebar)
Pinned left sidebar with section icons. Active section highlighted with
a 2px left border in primary green. Sections:
- Dashboard (grid icon)
- Impact (shield icon)
- Graph (network icon)
- Ledger (ledger/book icon)
- Compliance (checkmark icon)
- Verify (play icon)
- Settings (gear icon)

## Do's and Don'ts

- **Do** use dense information layouts — developers scan, they don't read
- **Do** show risk levels visually (color + icon + text) on every item
- **Do** make transaction IDs, commit hashes, and paths selectable/copyable
- **Do** use keyboard shortcuts for navigation (j/k to move through lists)
- **Don't** use light theme as default — this is a terminal-adjacent tool
- **Don't** use marketing language — labels should be precise ("3 HIGH risk
  changes" not "Several potentially impactful modifications")
- **Don't** hide data behind multiple clicks — every view should pass
  the "can I scan this in 3 seconds" test
- **Don't** use circular/spiral graph layouts for the KG — force-directed
  with physics is the expected developer tool paradigm
- **Don't** show loading spinners for data that's already loaded in the
  CLI cache — the UI should feel instant because all data is local

## Specific View Requirements

### Dashboard
- Top row: 4 metric cards (Pending Txns, Unaudited Drift, Risk Level, Last Verify)
- Main area: chronological change feed, filterable by repo/risk/category
- Right sidebar: hotspot top-5, verification pass/fail sparkline, bus factor

### Impact Explorer
- Search bar with filter chips (date range, risk level, repo, author)
- Results as a table with expandable rows
- Expanded view: changed files list, risk reasons, temporal couplings,
  predicted test impacts, linked ADRs

### Knowledge Graph
- Full-screen D3.js force-directed graph
- Legend panel (node types + edge types)
- Search box for node names
- Click node → detail panel with file info, dependencies, tests
- Transaction mode: highlight the Affects edges for a specific tx

### Ledger
- Table with columns: Timestamp, Category, Entity, Risk, Summary, Signed
- Filters: date range, category, entity, signer
- Click row → full detail: commit msg hash, signature, public key,
  related files, related tickets, linked ADR exports
- ADR tab: hierarchical tree of ADRs with supersession chains

### Compliance Hub
- Summary cards: Total signed transactions, last audit date, oldest
  unaddressed ADR, signature validity %
- Export button: generates SOC2 evidence ZIP
- Signature verification: shows valid/invalid/skipped counts per entry

### Settings
- Config file viewer with syntax highlighting and secret redaction
- LLM backend dropdown (local / Ollama Cloud / Gemini)
- Verification step editor (add/remove/reorder steps)
- MCP server status and restart button
```

---

## 7. Appendix: Key Files for Frontend Development

### REST API Surface (to be built into `changeguard web`)

The web UI backend exposes these endpoints from the existing CozoDB + SQLite
data. All return JSON.

```
GET  /api/v1/dashboard            # Summary metrics + recent feed
GET  /api/v1/changes              # Impact packet history, paginated
GET  /api/v1/changes/:id          # Single impact packet detail
GET  /api/v1/graph                # Knowledge graph nodes + edges
GET  /api/v1/graph/search?q=     # Search graph nodes
GET  /api/v1/ledger/entries       # Transaction table, paginated
GET  /api/v1/ledger/entries/:id   # Single transaction detail
GET  /api/v1/ledger/adrs          # ADR list with supersession
GET  /api/v1/verify/history       # Verification pass/fail trend
GET  /api/v1/verify/health        # Current health status
GET  /api/v1/hotspots             # Current hotspot rankings
GET  /api/v1/compliance/export    # SOC2 evidence ZIP download
GET  /api/v1/settings/config      # Config (redacted)
```

### Key Data Models (already exist in Rust)

- `ImpactPacket` — The core change analysis result (risk, files, couplings)
- `LedgerEntry` — Signed transaction record
- `Node` / `Edge` — Knowledge graph entities
- `Hotspot` — File hotspot score
- `VerificationResult` — Per-step pass/fail with timing
- `AdrEntry` — Architecture decision record with supersession

### Existing CLI Commands That Map to Web UI Actions

| Web UI Action | Equivalent CLI Command |
|---|---|
| View change feed | `changeguard ledger audit --json` |
| Search transactions | `changeguard ledger search --json` |
| View impact detail | `changeguard impact --json` |
| Explore graph | `changeguard viz` (HTML) |
| View hotspots | `changeguard hotspots --json` |
| Check verification health | `changeguard verify --health` |
| Export compliance | `changeguard ledger audit --json --sections` |
| View config | `changeguard config view --json` |
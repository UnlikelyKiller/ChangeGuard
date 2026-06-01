---
title: AI-Brains Project Skill
description: How to build, develop, and operate the AI-Brains local-first memory vault.
category: devops
version: 2.1.0
---

# AI-Brains Project Skill

## What This Is

AI-Brains is a local-first, event-sourced memory vault with:
- **SQLCipher** encrypted SQLite database (`vault.db`)
- **FTS5** full-text search over conversation turns and ingested documents
- **Semantic search** via stored embeddings (nomic-embed-text on port 8083)
- **Nightly summarization** job for session compaction and hierarchical synthesis
- **ChangeGuard bridge** for safety signals (HOTSPOT, DECISION, CONSTRAINT) and code symbol ingestion into recall (T70)
- **Live graph projection** layer — graph updated automatically on every event append (T69)
- **Graph-augmented recall** — top hits are score-boosted by their graph neighbors (T66)
- **Code symbol recall** — functions and routes from ChangeGuard are ingested during nightly and surface via `ai-brains recall` (T70)
- **Rust workspace** with ~15 crates under `crates/`

## Where It Lives

| Component | Location |
|-----------|----------|
| Source code | `C:\dev\AI-Brains` |
| Cargo binary | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` |
| Vault database | `C:\dev\ai-brains\vault.db` |
| Conductor tracks | `C:\dev\AI-Brains\conductor\tracks\` |
| Obsidian vault | `C:\Users\RyanB\Documents\Hermes\` |

## Build Requirements

### Windows (Primary Target — run from WSL)

```bash
# Install MinGW-w64 toolchain
sudo apt-get install -y gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64

# Configure Cargo linker (~/.cargo/config.toml)
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"

# Build with graph feature (required for graph CLI commands)
cargo build --features graph -p ai-brains-cli
```

### Linux Native Build

```bash
cargo build --release -p ai-brains-cli
```

## Common Commands

### Recall (Memory Retrieval)

```bash
# FTS5 keyword recall
ai-brains recall "GPU driver fix" --limit 5

# Semantic recall (requires Ollama + nomic-embed-text running on port 8083)
ai-brains recall "authentication flow" --semantic --limit 5

# Graph-boosted recall (boosts scores of graph-neighbor memories)
ai-brains recall "login handler" --semantic --graph-boost 0.1 --limit 5

# Project-scoped recall
ai-brains recall "query" --project-id <id>

# Control graph traversal depth (default 1)
ai-brains recall "query" --graph-boost 0.2 --graph-hop-depth 1
```

### Graph Queries (requires `--features graph` build)

The graph is updated **automatically** after every event append (T69 live hook). Manual `graph rebuild` is only needed after schema changes or corruption.

```bash
# 1-hop neighbors of a memory (all edge labels, both directions)
ai-brains graph neighbors <memory_id>

# Recursive SYNTHESIZED_FROM chain (what was this summary built from?)
ai-brains graph hierarchy <memory_id>

# All memories recalled in a session via graph edges
ai-brains graph session <session_id>

# Graph health check — current node/edge counts
ai-brains graph update

# Full resync from event log (recovery only — slow on large vaults)
ai-brains graph rebuild
```

### Vault Operations

```bash
# Initialize a new vault
ai-brains init --vault-path C:\dev\ai-brains\vault.db

# Ingest a file or directory
ai-brains ingest --vault-path C:\dev\ai-brains\vault.db --project-id <ID> --path C:\path\to\notes

# Preflight (project-scoped safety signals: HOTSPOT, DECISION, CONSTRAINT)
ai-brains preflight --vault-path C:\dev\ai-brains\vault.db --project-id <ID> --summary

# Nightly pipeline (summarization, embeddings, synthesis, symbol ingestion)
ai-brains nightly --vault-path C:\dev\ai-brains\vault.db

# Project alias resolution
ai-brains project resolve "Newton"
ai-brains project detect   # auto-detects from current git repo
```

### CI Gate (T71 — fully operational on Windows)

Run the full gate or use the verification script:

```powershell
# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit

# Or use the script (checks tool presence + versions, then runs gate)
.\scripts\dev-check.ps1

# Check tools only, skip running the gate
.\scripts\dev-check.ps1 --check-only
```

Required tool versions (see `Docs/ci-tooling.md` for install commands):

| Tool | Min Version |
|------|-------------|
| `cargo-nextest` | 0.9.137 |
| `cargo-deny` | 0.19.4 |
| `cargo-audit` | 0.22.1 |

### Testing

```bash
cargo nextest run --workspace
cargo test -p ai-brains-store
cargo build --features graph -p ai-brains-cli   # verify graph feature compiles
```

### Conductor Workflow

All tracks follow the Conductor pattern:
1. **Spec** in `conductor/tracks/trackTNN-<name>/spec.md`
2. Register in `conductor/conductor.md`
3. Implement → test → lint (CI gate must pass)
4. Update registry status to Complete

Current track registry: T61–T71 (all complete). See `conductor/conductor.md`.

## Key Architecture

### Crate Layout

| Crate | Purpose |
|-------|---------|
| `ai-brains-core` | IDs, privacy types, session model |
| `ai-brains-events` | Immutable event definitions and payload types |
| `ai-brains-store` | SQLCipher event store + read projections |
| `ai-brains-path` | Windows/WSL/UNC path normalization |
| `ai-brains-capture` | CLI/daemon capture pipeline |
| `ai-brains-retrieval` | FTS5 + semantic search + graph-augmented recall |
| `ai-brains-graph` | GraphProjector, SqliteGraphBackend, CozoProxy bridge |
| `ai-brains-brain` | NightlyService, MemorySynthesizer, EmbeddingService |
| `ai-brains-cli` | Main CLI binary + LiveGraphHook (T69) |
| `ai-brainsd` | Background daemon |

### Graph Architecture (T66–T69)

The graph is stored in SQLite (`graph_node`, `graph_edge` tables) and optionally mirrored to ChangeGuard's CozoDB via `CozoProxyBackend`. Events are projected to graph nodes/edges by `GraphProjector`.

**Live hook (T69):** Every `EventStore::append_event()` call in the CLI automatically applies the event to the graph projector and flushes. No manual rebuild needed for routine operations.

**Edge types in the graph:**
- `IN_SESSION` — turn → session
- `IN_PROJECT` — session → project
- `RECALLS` — session → memory (created when `recall` is called, T67)
- `SYNTHESIZED_FROM` — summary → source memories (created during nightly, T68)
- `CONFLICTS_WITH` — conflict → memory
- `PART_OF_RECIPE` — recipe node

### Recall vs Preflight vs ChangeGuard Search

| Goal | Command |
|------|---------|
| Find past decisions, session memories, or code symbols | `ai-brains recall --semantic` |
| Get project-scoped safety signals before editing | `ai-brains preflight --project-id <id>` |
| Find a live function or endpoint by name | `changeguard search "functionName"` |
| Natural language code queries | `changeguard ask "find all GET handlers"` |
| Blast radius of a change | `changeguard scan --impact` |

> **As of T70:** `ai-brains recall` also returns code symbols (functions, routes) ingested from ChangeGuard during nightly. A single recall query is sufficient for most questions about past decisions and code structure.

### Nightly Pipeline (what runs automatically)

1. Antigravity session import
2. Session summarization (batch-limited via `AI_BRAINS_NIGHTLY_BATCH` env var)
3. Hierarchical memory synthesis (`MemorySynthesized` events → SYNTHESIZED_FROM edges)
4. WAL checkpoint
5. Embedding backfill (50 memories without embeddings)
6. Stale embedding refresh (10 memories older than 30 days)
7. MADR ingestion from ChangeGuard ledger
8. ChangeGuard symbol index refresh + symbol ingestion into recall (T70)

## Windows-Specific Notes

1. **Paths**: Use forward slashes or escaped backslashes in PowerShell
2. **PowerShell encoding**: Always set `$OutputEncoding = [System.Text.Encoding]::UTF8`
3. **Cargo config**: `~/.cargo/config.toml` required for `x86_64-pc-windows-gnu`
4. **Nightly batch limit**: Set `AI_BRAINS_NIGHTLY_BATCH=50` to cap sessions processed per run

## Safety & Privacy

- **Privacy filter** runs on all ingested content — never expose API keys, passwords, or tokens
- **Redaction** automatically masks `sk-...`, `ghp_...`, `SG.*` patterns
- **Pinned memories** (HOTSPOT, CONSTRAINT, DECISION) are promoted to preflight context
- **Graph failures are non-fatal** — primary event append always succeeds even if graph projection fails

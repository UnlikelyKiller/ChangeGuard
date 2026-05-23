---
name: changeguard
description: Use this skill when working in a repository initialized with `.changeguard/` and the task involves code edits, reviews, impact/risk analysis, verification planning, drift handling, ledger provenance, or deciding what tests to run. Before meaningful edits, run ChangeGuard scan/impact; after edits, run verification and report unresolved drift or ledger state.
---

# ChangeGuard

Use ChangeGuard as the local safety layer and engineering intelligence engine for code changes. It provides impact analysis, hotspot and temporal-coupling signals, verification planning, and transactional provenance.

## Core Capabilities

- **Search & Discovery**: High-performance regex (Tantivy), precise LSP navigation (SCIP), and conceptual semantic search (local embeddings) with parallel HNSW retrieval.
- **Knowledge Graph**: Durable, billion-edge relational and vector storage (CozoDB-redux/Sled) with native code-aware tokenization (Tree-Sitter).
- **Impact Analysis**: Deep "blast radius" analysis across 20+ specialized providers (Infra, Contracts, Observability, Temporal).
- **Cryptographic Provenance**: Mathematical proof of intent via Ed25519 signing of every ledger entry. Offline verification via `verify --signatures`.
- **Intent Capture TUI**: Interactive terminal UI for auditing and refining LLM-drafted intent payloads during the git commit process.
- **Real-time Sync**: Incremental Knowledge Graph updates, AST re-parsing, and code-aware symbol indexing via the `watch` command.
- **Predictable Verification**: Bayesian test reordering and CI failure prediction.
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

## Repository Configuration

ChangeGuard's `.changeguard/rules.toml` and `.changeguard/config.toml` are
repo-local policy, not portable defaults. When installing or copying this skill
into another repository, review and update:

- `required_verifications`: use commands that actually exist in that repo
  rather than aliases such as `lint`, `test`, or `build` unless the repo defines
  those commands.
- `verify.default_timeout_secs`: set a timeout that fits the repo's slowest
  expected verification command.
- `protected_paths`: keep enforcement scoped to paths that make sense for the
  repository.

If `changeguard verify` fails with "Command not found" or times out while the
same command passes manually, fix the repo-local config before treating it as a
code failure.

## Dependency Alert Workflow

For Dependabot or audit findings:

- Identify whether the vulnerable crate is direct or transitive with
  `cargo tree -i <crate>@<version>`.
- If the vulnerable crate is transitive through a direct dependency, prefer
  upgrading the direct dependency over adding a downstream patch.
- If the vulnerable path enters through a git dependency, verify whether the
  upstream fix is visible to downstream consumers. Workspace-level
  `[patch.crates-io]` entries in the dependency repository are not transitive.
- Record external remediation handoffs in a conductor track when another repo
  owns the durable fix.
- After dependency changes, run focused dependency checks plus `changeguard
  verify`.

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
- If a command reports that the index is `[STALE]`, you can append the `--auto-index` flag to commands like `search`, `ask`, `hotspots`, or `dead-code` to automatically refresh it before executing.
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

### Git Hook Lifecycle (Milestone O)

ChangeGuard uses a two-phase commit lifecycle to ensure zero phantom records:
1. **`commit-msg`**: Launches the TUI to capture intent. Creates a `PENDING` transaction and a sidecar file.
2. **`post-commit`**: Automatically promotes the `PENDING` transaction to `COMMITTED` once the Git commit is finalized. If the Git commit fails, the record remains pending or is safely rolled back on the next attempt.

### Cryptographic Security

If `intent.require_signing = true` is set in `.changeguard/config.toml`, all ledger entries must be signed by the developer's local Ed25519 key (generated during `init`).

To verify the integrity of the entire ledger:
```bash
changeguard verify --signatures
```
This performs an offline mathematical validation of every record against its signature and public key.

## Reasoning Rules

- If temporal coupling is above 70% for an unchanged file, inspect that file.
- If hotspots are reported, bias verification toward those files first.
- If KG reachability identifies downstream nodes, inspect them before finalizing.
- Treat hooks and CI gates as enforcement. Treat this skill as guidance.

## Maintenance & Upgrades

To keep your ChangeGuard environment synchronized with the latest engine features:

```bash
# Safely migrate repository state (clears indices, preserves ledger)
changeguard update --migrate --force

# Rebuild indices after migration
changeguard index --semantic
```

## Working On ChangeGuard Itself

After changing ChangeGuard source code, you can use the built-in update command to reinstall the global binary:

```bash
changeguard update --binary
```

Alternatively, run manually from the source root:

```bash
cargo install --path .
```

## References

- Command details: `references/commands.md` (includes ledger, impact, dead-code, viz-server, doc generation, watch)
- Install fallback: `references/install.md`
- Architecture/internal notes: `references/internals.md`

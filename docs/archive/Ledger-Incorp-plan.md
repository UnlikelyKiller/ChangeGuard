# ChangeGuard Ledger Incorporation Plan

## Overview

This document defines the implementation roadmap for incorporating Project Ledger's transactional architectural memory into ChangeGuard. The goal is a **single Rust binary** that combines ChangeGuard's change-intelligence (impact analysis, risk scoring, verification, Gemini integration) with Ledger's change-provenance tracking (transaction lifecycle, tech stack enforcement, drift detection, audit trail, ADR export).

The result: one tool that detects change, scores impact, enforces constraints, records provenance, and verifies correctness — end to end.

This plan is written for implementation by AI coding agents and humans working together. It follows the same discipline as the ChangeGuard Implementation Plan v1: stable phase boundaries, conservative defaults, deterministic behavior, explicit failure handling, pinned dependency guidance.

---

## 0. Design Rationale

### 0.1 Why Incorporate Rather Than Integrate

The AI feedback from projects using both tools identified a core problem: **two halves of the same brain, disconnected**. ChangeGuard tells you what might break; Ledger records what changed and why. But they share no state, no linking keys, and no unified workflow. The result is governance theater — agents go through the motions of both tools without the outputs actually changing decisions.

Incorporating into one binary solves this:

1. **Shared state**: A ledger transaction can reference an impact packet. A verify run can gate a commit.
2. **Unified workflow**: `scan → ledger start → edit → impact → ledger commit` is one flow, not two disconnected rituals.
3. **Reduced ceremony**: Ledger's `atomic_change` pattern (start+commit in one call) already matches ChangeGuard's one-shot scan style. No need for heavy ceremony on small changes.
4. **Single config**: One `.changeguard/config.toml` instead of `.changeguard/config.toml` + `.project-ledger/config.json`.
5. **Single state store**: One `ledger.db` under `.changeguard/state/` instead of two separate databases.

### 0.2 What We Keep From Ledger

| Capability | Why |
|---|---|
| Transaction lifecycle (start → commit → rollback) | Provenance tracking, the core value proposition |
| UNAUDITED drift detection | Independent verification that agents followed the protocol |
| Tech stack enforcement | Prevents architectural violations before code is written |
| Commit validators | Configurable pass/fail gates at commit time |
| FTS5 search over history | Makes the audit trail actually queryable |
| ADR export (MADR format) | Standardized architectural decision records |
| Artifact reconciliation | Auto-suggests what changed via `git diff` |
| Breaking change annotation | Explicit downstream impact flagging |
| Verification status per transaction | Links verification evidence to specific changes |

### 0.3 What We Drop From Ledger

| Capability | Why |
|---|---|
| MCP server transport | ChangeGuard is a CLI-first tool; MCP is an anti-goal per Plan.md v1 |
| Node.js/TypeScript runtime | Everything must be Rust |
| REST API sidecar (port 9183) | HTTP service layer is an anti-goal per Plan.md v1 |
| Metrics server (Prometheus) | Premature for CLI-first; add later if needed |
| Webhook emitter | Fire-and-forget HTTP is fragile; add later as opt-in |
| Forge integration (GitHub issue creation) | Not core to provenance tracking |
| Open Brain direct DB writes | Decouple lesson storage; store lessons in ChangeGuard's own DB |
| `should_skip_ledger` (LLM-gated) | Requires LLM call to decide if change is trivial; incompatible with CLI-first model |
| TOML entry file export | Nice-to-have; defer to polish phase |
| `entities` array (multi-file) | Known UNIQUE constraint bug per Ledger docs; single entity per transaction is cleaner |

### 0.4 What We Redesign

| Original (Ledger) | Redesign (ChangeGuard) | Why |
|---|---|---|
| `.project-ledger/` state dir | `.changeguard/` (existing) | Single state root |
| `config.json` | `config.toml` (existing) | Consistent format |
| Category: `API_ENDPOINT`, `DB_SCHEMA`, etc. | Unified: `ARCHITECTURE`, `FEATURE`, `BUGFIX`, `REFACTOR`, `INFRA`, `TOOLING`, `DOCS`, `CHORE` | Ledger had two category systems; one is enough |
| `source: AGENT | WATCHER` | `source: CLI | WATCHER` | CLI, not MCP |
| `session_id` for stale detection | `session_id` based on process startup timestamp | Same concept, Rust-native |
| 8-char UUID truncation (old bug) | Full UUID display always | Avoid the copy-paste friction |
| Separate `watcher_patterns` DB table | Three-source model: hardcoded + config + DB (for `ledger register --rule-type WATCHER`) | Runtime-registered patterns need persistence beyond config; DB table is required |
| `manage_transaction` (deprecated) | Not ported | Use dedicated atomic tools |

### 0.5 Feedback-Driven Improvements

These come directly from AI agents who used both tools on real projects:

1. **Lightweight note mode**: `ledger note --entity X --summary "..."` for docs-only changes, auto-defaults verification_basis to `manual_inspection`. Avoids ceremony for trivial changes.
2. **Auto-reconcile**: `--auto-reconcile` flag on `atomic` and `commit` that automatically resolves watcher drift for the same entity. One operation instead of two.
3. **Drift deduplication**: Same file modified multiple times in one session shows as one entry with a count, not N UUIDs.
4. **Active session vs stale drift**: Files belonging to a pending transaction are shown separately from orphaned changes in `status`.
5. **Positional search**: `ledger search "query"` instead of requiring `--query "query"`.
6. **Transaction resume**: `ledger resume` finds the most recent PENDING transaction for the current context, reducing UUID hunting.
7. **Fuzzy tx_id matching**: Truncated UUIDs that are unique within the pending set are accepted, with disambiguation prompt if ambiguous.
8. **Verification-as-gate**: ChangeGuard's `verify` results can be auto-attached to a `ledger commit`, and `commit` can be configured to require verification pass.
9. **Impact-to-ledger bridge**: `impact --ledger-start` auto-opens ledger transactions for entities identified in the impact report.
10. **Configurable verify commands**: Already implemented in ChangeGuard via `config.verify.steps`.

---

## 1. Core Implementation Principles

### 1.1 Non-Negotiable Principles (Carried Forward)

1. **Single-binary Rust CLI first**.
2. **Repo-local state by default** under `.changeguard/`.
3. **Conservative, deterministic behavior** over speculative automation.
4. **Windows-first execution quality**.
5. **No `unwrap`/`expect` in production paths**.
6. **Typed errors with `thiserror` + `miette::Diagnostic` for user-facing, `anyhow` for internal infrastructure**. Every subsystem defines its own error enum (e.g., `LedgerError`, `ConfigError`, `StateError`).
7. **Graceful degradation** when partial features fail.
8. **Safe rebuildability** of all generated local state.
9. **Data classification**. Not all state under `.changeguard/` is the same kind of data. See Section 1.1.1.

### 1.1.1 Data Classification: Derived State vs Durable User Data

Current ChangeGuard treats all `.changeguard/` state as rebuildable derived cache. The ledger introduces durable user data that must not be casually deleted. These are fundamentally different:

| Category | Examples | Rebuildable? | `reset` behavior |
|---|---|---|---|
| **Derived cache** | impact packets, scan indexes, verification results, snapshots, federated links | Yes — regenerated from repo + config | Deleted by default |
| **Durable user data** | transactions, ledger entries, tech stack rules, commit validators, lessons | No — authored by users, not derivable | Preserved by default |
| **Configuration** | config.toml, watcher patterns (config-defined) | N/A — user-maintained | Preserved always |

`changeguard reset` behavior:
- Default (`changeguard reset`): deletes derived cache only, preserves `ledger.db` transactions and rules.
- `changeguard reset --include-ledger`: also deletes `ledger.db` (requires `--force` or interactive confirmation).
- `changeguard reset --all`: deletes everything including config (existing behavior, requires `--force`).

This resolves the conflict between "all generated state is rebuildable" (Principle 8) and "immutable audit trail" (Principle 12). Derived state is rebuildable; user-authored provenance is not.

### 1.2 Ledger-Specific Principles

14. **Transactions are the unit of provenance**. Every structural change should flow through a transaction.
15. **Trust-and-verify model**. Agents self-report via transactions; the watcher independently detects drift. Both are first-class.
16. **Enforcement is opt-in, not default**. Tech stack rules and commit validators are only active when configured. Empty config = no enforcement.
17. **Immutable audit trail**. Once a transaction is COMMITTED, it cannot be modified. Corrections are new transactions.
18. **Provenance over perfection**. A transaction with `unverified` status is better than no transaction at all.

### 1.3 Explicit Anti-Goals

Do not introduce:

- MCP server transport
- HTTP/REST API layer
- Cloud sync or telemetry
- Background daemon beyond the existing watch mode
- Autonomous commit/rebase flows
- Multi-entity transactions (single entity per transaction; use `operation_id` to group related single-entity transactions)

---

## 2. Architecture Boundaries

The implementation must preserve separation between these subsystems (existing from Plan.md v1, plus new):

| # | Subsystem | Role |
|---|---|---|
| 1 | CLI routing | Command dispatch, arg parsing |
| 2 | Platform detection | OS/shell/env/path handling |
| 3 | Repo-local state management | SQLite, layout, migrations |
| 4 | Git scanning and diff analysis | Status, history, diff |
| 5 | Watcher and debounce batching | File event detection |
| 6 | Language-aware indexing | Symbol/import extraction |
| 7 | Impact and risk scoring | Blast radius, temporal coupling |
| 8 | Policy/rules evaluation | Protected paths, verification rules |
| 9 | Verification planning and execution | Test/lint command orchestration |
| 10 | Gemini prompt generation | Context assembly and invocation |
| **11** | **Transaction lifecycle** | **Start, commit, rollback, adopt** |
| **12** | **Drift detection** | **Watcher → UNAUDITED transaction bridge** |
| **13** | **Tech stack enforcement** | **Rules, validators, category mapping** |
| **14** | **Audit trail and search** | **FTS5, ADR export, history queries** |
| **15** | **Artifact reconciliation** | **Git diff → entity suggestion** |

---

## 3. Expanded Repository Layout

```text
changeguard/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── docs/
│   ├── Plan.md
│   ├── Ledger-Incorp-plan.md
│   ├── Engineering.md
│   └── ChangeGuard/
│       └── skill.md
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── lib.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── doctor.rs
│   │   ├── scan.rs
│   │   ├── watch.rs
│   │   ├── impact.rs
│   │   ├── verify.rs
│   │   ├── ask.rs
│   │   ├── reset.rs
│   │   ├── hotspots.rs
│   │   ├── federate.rs
│   │   │   ** ledger.rs          -- Transaction lifecycle commands
│   │   │   ** ledger_commit.rs
│   │   │   ** ledger_rollback.rs
│   │   │   ** ledger_atomic.rs
│   │   │   ** ledger_status.rs
│   │   │   ** ledger_reconcile.rs
│   │   │   ** ledger_adopt.rs
│   │   │   ** ledger_search.rs
│   │   │   ** ledger_audit.rs
│   │   │   ** ledger_stack.rs
│   │   │   ** ledger_register.rs
│   │   │   ** ledger_adr.rs
│   │   │   ** ledger_scaffold.rs
│   │   │   ** ledger_artifacts.rs
│   │   │   ** ledger_resume.rs
│   │   │   ** ledger_note.rs
│   ├── config/
│   │   ├── model.rs           -- Extended with LedgerConfig
│   │   ├── load.rs
│   │   ├── validate.rs
│   │   ├── defaults.rs
│   │   └── error.rs
│   ├── ledger/                 ** NEW subsystem
│   │   ├── mod.rs
│   │   ├── types.rs           -- Category, ChangeType, VerificationStatus, EntryType, etc.
│   │   ├── error.rs           ** NEW: LedgerError enum (thiserror + miette::Diagnostic)
│   │   ├── db.rs              ** NEW: Ledger-specific SQLite operations (shares Connection with StorageManager)
│   │   ├── transaction.rs     -- Transaction lifecycle manager
│   │   ├── drift.rs           -- Watcher → UNAUDITED bridge
│   │   ├── enforcement.rs     -- Tech stack validation at start_change
│   │   ├── validators.rs     -- Shell-command validators at commit
│   │   ├── search.rs          -- FTS5 search engine
│   │   ├── adr.rs             -- MADR-format ADR exporter
│   │   ├── reconcile.rs      -- Artifact reconciliation (git diff)
│   │   ├── lesson.rs          -- Lesson/convention storage (Open Brain analog)
│   │   └── session.rs         -- Session tracking, stale detection
│   ├── state/
│   │   ├── layout.rs          -- Extended for ledger paths
│   │   ├── storage.rs         -- Extended with ledger tables
│   │   ├── migrations.rs      -- Extended with new migrations
│   │   ├── reports.rs
│   │   ... (existing modules unchanged)
│   └── output/
│       ├── human.rs           -- Extended with ledger output
│       ... (existing modules unchanged)
└── tests/
    ├── cli_scan.rs
    ├── cli_impact.rs
    ├── cli_ask.rs
    ├── temporal_coupling.rs
    ** ledger_lifecycle.rs      -- Transaction start/commit/rollback
    ** ledger_drift.rs          -- Drift detection and reconciliation
    ** ledger_enforcement.rs    -- Tech stack rules
    ** ledger_search.rs        -- FTS5 search
    ** ledger_adr.rs           -- ADR export
    ** ledger_artifacts.rs     -- Artifact reconciliation
```

---

## 4. Data Model

### 4.1 New SQLite Tables

All new tables are added to the existing `.changeguard/state/ledger.db` via incremental migrations. The existing tables (snapshots, batches, changed_files, verification_runs, verification_results, symbols, federated_links, federated_dependencies) remain untouched.

#### `transactions`

```sql
CREATE TABLE IF NOT EXISTS transactions (
    tx_id              TEXT PRIMARY KEY,     -- Full UUID v4
    operation_id       TEXT,                 -- Groups related single-entity transactions (set at start or via --operation-id)
    status             TEXT NOT NULL,        -- PENDING | COMMITTED | ROLLED_BACK | UNAUDITED | RECONCILED
    category           TEXT NOT NULL,        -- ARCHITECTURE | FEATURE | BUGFIX | REFACTOR | INFRA | TOOLING | DOCS | CHORE
    entity             TEXT NOT NULL,        -- File path or component name
    entity_normalized  TEXT NOT NULL,        -- Platform-aware normalization: forward-slash, relative to workspace root. See Path Normalization for case-folding rules.
    planned_action     TEXT,                 -- Description at start_change time
    session_id         TEXT NOT NULL,        -- Process startup timestamp as correlation ID
    source             TEXT NOT NULL DEFAULT 'CLI',  -- CLI | WATCHER
    started_at         TEXT NOT NULL,        -- ISO 8601 UTC
    resolved_at        TEXT,                 -- ISO 8601 UTC, null while PENDING
    detected_at        TEXT,                 -- ISO 8601 UTC for UNAUDITED (watcher timestamp; intentionally TEXT not INTEGER, unlike Ledger which uses Unix epoch)
    drift_count        INTEGER DEFAULT 1,   -- Number of watcher hits for this entity (deduplication counter)
    first_seen_at      TEXT,                 -- ISO 8601 UTC, first watcher detection time for UNAUDITED
    last_seen_at       TEXT,                 -- ISO 8601 UTC, most recent watcher detection time for UNAUDITED
    issue_ref          TEXT,                 -- Optional issue/ticket reference
    change_type        TEXT,                 -- CREATE | MODIFY | DELETE | DEPRECATE (set at commit)
    summary            TEXT,                 -- Technical summary (set at commit)
    reason             TEXT,                 -- Architectural intent (set at commit)
    is_breaking        INTEGER DEFAULT 0,   -- 1 = downstream impact
    verification_status TEXT,                -- verified | unverified | partially_verified | failed
    verification_basis TEXT,                -- tests | build | lint | runtime | manual_inspection | inferred
    outcome_notes      TEXT,                -- Notes on the outcome
    snapshot_id        INTEGER REFERENCES snapshots(id),  -- Links to ChangeGuard impact packet
    tree_hash          TEXT                  -- git HEAD commit + dirty-tree digest at commit time (binds verification to exact repo state)
);

CREATE INDEX IF NOT EXISTS idx_transactions_entity_status ON transactions(entity_normalized, status);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
CREATE INDEX IF NOT EXISTS idx_transactions_session_id ON transactions(session_id);
CREATE INDEX IF NOT EXISTS idx_transactions_operation_id ON transactions(operation_id);
```

**Operation ID.** Each transaction still has exactly one entity (single-entity rule preserved). An `operation_id` groups related transactions that belong to the same logical change (e.g., an `impact --ledger-start` that opens 5 transactions). This avoids multi-entity complexity while giving `ledger status` and `ledger audit` the ability to show "3 files changed as part of operation X." If no `operation_id` is provided, each transaction is independent.

**Tree Hash Binding.** At commit time, `tree_hash` captures `git rev-parse HEAD` + a dirty-tree digest (hash of `git diff --stat` output). This binds the verification evidence to the exact repo state. If the working tree has changed since verification ran, `ledger commit` warns that the tree hash differs from the verification snapshot.

**Field Validity by Status.** Not all fields are populated at every lifecycle stage. Agents and query authors must not assume nullable fields are present:

| Field | PENDING | COMMITTED | ROLLED_BACK | UNAUDITED | RECONCILED |
|---|---|---|---|---|---|
| `tx_id` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `status` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `category` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `entity` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `planned_action` | ✓ | ✓ | — | — | — |
| `session_id` | ✓ | ✓ | ✓ | ✓* | ✓* |
| `started_at` | ✓ | ✓ | ✓ | — | — |
| `detected_at` | — | — | — | ✓ | ✓ |
| `source` | `CLI` | `CLI` | `CLI` | `WATCHER` | `WATCHER` |
| `resolved_at` | — | ✓ | ✓ | ✓ | ✓ |
| `change_type` | — | ✓ | — | — | ✓ |
| `summary` | — | ✓ | — | — | ✓ |
| `reason` | — | ✓ | ✓ | — | ✓ |
| `is_breaking` | — | ✓ | — | — | ✓ |
| `verification_status` | — | ✓ | — | — | ✓ |
| `verification_basis` | — | ✓ | — | — | ✓ |
| `outcome_notes` | — | optional | optional | — | optional |
| `snapshot_id` | — | optional | — | — | optional |
| `tree_hash` | — | ✓ | — | — | ✓ |
| `operation_id` | optional | optional | optional | — | optional |
| `drift_count` | — | — | — | ✓ | ✓ |
| `first_seen_at` | — | — | — | ✓ | ✓ |
| `last_seen_at` | — | — | — | ✓ | ✓ |
| `issue_ref` | optional | optional | optional | — | optional |

\* UNAUDITED/RECONCILED: `session_id` is the watcher's session, not the original author's.

#### `ledger_entries`

```sql
CREATE TABLE IF NOT EXISTS ledger_entries (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_id              TEXT NOT NULL REFERENCES transactions(tx_id),
    operation_id       TEXT,                  -- Copied from transactions.operation_id for query convenience
    category           TEXT NOT NULL,
    entry_type         TEXT NOT NULL DEFAULT 'IMPLEMENTATION',  -- IMPLEMENTATION | ARCHITECTURE | LESSON
    entity             TEXT NOT NULL,
    entity_normalized  TEXT NOT NULL,
    change_type        TEXT NOT NULL,
    summary            TEXT NOT NULL,
    reason             TEXT NOT NULL,
    is_breaking        INTEGER DEFAULT 0,
    committed_at       TEXT NOT NULL,         -- ISO 8601 UTC
    issue_ref          TEXT,
    trace_id           TEXT,                  -- Optional distributed tracing / correlation ID
    origin             TEXT NOT NULL DEFAULT 'LOCAL',  -- LOCAL | SIBLING (federated)
    snapshot_id        INTEGER REFERENCES snapshots(id),
    tree_hash          TEXT                   -- Copied from transactions.tree_hash for query convenience
);

CREATE INDEX IF NOT EXISTS idx_ledger_entries_entity ON ledger_entries(entity_normalized);
CREATE INDEX IF NOT EXISTS idx_ledger_entries_category ON ledger_entries(category);
CREATE INDEX IF NOT EXISTS idx_ledger_entries_committed_at ON ledger_entries(committed_at);
CREATE INDEX IF NOT EXISTS idx_ledger_entries_operation_id ON ledger_entries(operation_id);
```

#### `ledger_fts`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS ledger_fts
    USING fts5(entity, summary, reason, content=ledger_entries, content_rowid=id);

-- FTS5 content-sync triggers (required for external-content FTS5 tables)
CREATE TRIGGER IF NOT EXISTS ledger_fts_ai AFTER INSERT ON ledger_entries BEGIN
    INSERT INTO ledger_fts(rowid, entity, summary, reason) VALUES (new.id, new.entity, new.summary, new.reason);
END;
CREATE TRIGGER IF NOT EXISTS ledger_fts_ad AFTER DELETE ON ledger_entries BEGIN
    INSERT INTO ledger_fts(ledger_fts, rowid, entity, summary, reason) VALUES ('delete', old.id, old.entity, old.summary, old.reason);
END;
CREATE TRIGGER IF NOT EXISTS ledger_fts_au AFTER UPDATE ON ledger_entries BEGIN
    INSERT INTO ledger_fts(ledger_fts, rowid, entity, summary, reason) VALUES ('delete', old.id, old.entity, old.summary, old.reason);
    INSERT INTO ledger_fts(rowid, entity, summary, reason) VALUES (new.id, new.entity, new.summary, new.reason);
END;
```

#### `tech_stack`

```sql
CREATE TABLE IF NOT EXISTS tech_stack (
    category           TEXT PRIMARY KEY,     -- DATABASE | BACKEND_LANG | FRONTEND_FRAMEWORK | ORM | AUTH | TESTING | CI_CD | HOSTING
    name               TEXT NOT NULL,        -- e.g., SQLite, Rust, React
    version_constraint TEXT,                 -- e.g., >=1.70
    rules              TEXT NOT NULL DEFAULT '[]',  -- JSON array of strings or {rule, suggestion} objects
    locked             INTEGER DEFAULT 0,   -- 1 = cannot be overridden without explicit unlock
    status             TEXT DEFAULT 'ACTIVE',  -- ACTIVE | DEPRECATED | PROPOSED
    entity_type        TEXT DEFAULT 'FILE',  -- FILE | ABSTRACT
    registered_at      TEXT NOT NULL
);
```

#### `commit_validators`

```sql
CREATE TABLE IF NOT EXISTS commit_validators (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    category           TEXT NOT NULL,        -- Which transaction categories this applies to
    name               TEXT NOT NULL,        -- Human-readable name
    description        TEXT,                 -- Human-readable explanation
    executable         TEXT NOT NULL,        -- Program name (e.g., "cargo", "npx")
    args               TEXT NOT NULL,        -- JSON array of args, {entity} placeholder allowed in any arg (e.g., '["check","{entity}"]')
    timeout_ms         INTEGER DEFAULT 30000,
    glob               TEXT,                 -- File-type scope
    validation_level   TEXT DEFAULT 'ERROR', -- ERROR | WARNING
    enabled            INTEGER DEFAULT 1    -- 0 = disabled, 1 = active
);
```

**Why argv-style, not shell strings.** The existing verify runner (`src/verify/runner.rs`) already uses argv-style `PreparedStep { executable, args }` for direct process execution, with shell fallback only for metacharacter-containing commands. Storing validators as `executable` + `args` (JSON array) follows this pattern and avoids Windows shell quoting problems. The `{entity}` placeholder is substituted into individual args before execution, not passed through shell expansion.

#### `category_stack_mappings`

```sql
CREATE TABLE IF NOT EXISTS category_stack_mappings (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    ledger_category    TEXT NOT NULL,         -- Maps transaction categories to tech stack
    stack_category     TEXT NOT NULL REFERENCES tech_stack(category),
    glob               TEXT,                 -- Optional file-type scope
    description        TEXT
);
```

#### `service_dependencies`

```sql
CREATE TABLE IF NOT EXISTS service_dependencies (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    service_name       TEXT NOT NULL,
    depends_on         TEXT NOT NULL,
    dependency_type    TEXT NOT NULL,         -- API | SCHEMA | SHARED_LIB | PACKAGE
    description        TEXT
);
```

#### `deployments`

```sql
CREATE TABLE IF NOT EXISTS deployments (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    version            TEXT NOT NULL,
    environment        TEXT NOT NULL,
    git_sha            TEXT NOT NULL,
    deployed_at        TEXT NOT NULL,
    last_entry_id      INTEGER REFERENCES ledger_entries(id),
    notes              TEXT
);
```

#### `lessons`

```sql
CREATE TABLE IF NOT EXISTS lessons (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    category           TEXT NOT NULL,         -- GRAVEYARD | ENVIRONMENT | CONVENTION
    issue_context      TEXT NOT NULL,         -- What went wrong or what convention
    corrective_action  TEXT NOT NULL,         -- What to do instead
    confidence         TEXT DEFAULT 'medium', -- high | medium | low
    scope              TEXT DEFAULT 'GLOBAL',  -- GLOBAL | PROJECT_BOUND
    project_scope      TEXT,                  -- Project name if PROJECT_BOUND
    source             TEXT DEFAULT 'MANUAL',  -- MANUAL | LEDGER_HOOK (auto from --globalize)
    basis              TEXT DEFAULT 'inferred', -- verified_by_tests | observed_in_prod | inferred | speculative
    dependencies       TEXT,                  -- JSON: relevant packages and versions
    expires_at         TEXT,                  -- ISO 8601, optional hard expiry
    created_at         TEXT NOT NULL          -- ISO 8601 UTC
);

CREATE VIRTUAL TABLE IF NOT EXISTS lessons_fts
    USING fts5(issue_context, corrective_action, content=lessons, content_rowid=id);

-- FTS5 content-sync triggers for lessons
CREATE TRIGGER IF NOT EXISTS lessons_fts_ai AFTER INSERT ON lessons BEGIN
    INSERT INTO lessons_fts(rowid, issue_context, corrective_action) VALUES (new.id, new.issue_context, new.corrective_action);
END;
CREATE TRIGGER IF NOT EXISTS lessons_fts_ad AFTER DELETE ON lessons BEGIN
    INSERT INTO lessons_fts(lessons_fts, rowid, issue_context, corrective_action) VALUES ('delete', old.id, old.issue_context, old.corrective_action);
END;
CREATE TRIGGER IF NOT EXISTS lessons_fts_au AFTER UPDATE ON lessons BEGIN
    INSERT INTO lessons_fts(lessons_fts, rowid, issue_context, corrective_action) VALUES ('delete', old.id, old.issue_context, old.corrective_action);
    INSERT INTO lessons_fts(rowid, issue_context, corrective_action) VALUES (new.id, new.issue_context, new.corrective_action);
END;
```

#### `watcher_patterns`

```sql
CREATE TABLE IF NOT EXISTS watcher_patterns (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    glob               TEXT NOT NULL,         -- File glob pattern
    category           TEXT NOT NULL,         -- Maps to transaction category
    source             TEXT NOT NULL DEFAULT 'CONFIG',  -- CONFIG (from config.toml) | DB (runtime-registered) | DEFAULT (hardcoded)
    description        TEXT                  -- Human-readable explanation
);
```

### 4.2 Transaction Lifecycle

```
          start_change
  ┌──────► PENDING ──────────┐
  │                           │ commit_change
  │                     ┌─────▼──────┐
  │                     │  COMMITTED │───► ledger_entries (immutable)
  │                     └────────────┘
  │
  │  rollback_change
  ├──────► ROLLED_BACK
  │
  │  File watcher detects change
  │  with no matching PENDING tx
  └──────► UNAUDITED ───────┐
                             │ reconcile_unaudited
                       ┌─────▼──────┐
                       │ RECONCILED │───► ledger_entries (immutable, preserves watcher provenance)
                       └────────────┘
  Note: RECONCILED is distinct from COMMITTED because it carries different intent. COMMITTED means
  "a human proactively verified this change." RECONCILED means "a watcher detected this drift and
  a human acknowledged it after the fact." This distinction matters for audit queries like "show
  all changes that went through proper verification" vs "show all changes discovered retroactively."
  While `source: WATCHER` could theoretically replace this status, collapsing the two would lose
  the semantic signal in status-based queries that must filter for proactive verification.
                       └────────────┘

  adopt_transaction
  UNAUDITED ──► PENDING (recover stale transactions)
```

### 4.3 Category Model

Single category enum (not two systems like Ledger had):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Category {
    Architecture,
    Feature,
    Bugfix,
    Refactor,
    Infra,
    Tooling,
    Docs,
    Chore,
}
```

### 4.4 Change Type Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeType {
    Create,
    Modify,
    Deprecate,
    Delete,
}
```

### 4.5 Entry Type Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntryType {
    Implementation,
    Architecture,
    Lesson,
}
```

### 4.6 Verification Model

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Unverified,
    PartiallyVerified,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum VerificationBasis {
    Tests,
    Build,
    Lint,
    Runtime,
    ManualInspection,
    Inferred,
}
```

**Verification requirement rule**: Categories `ARCHITECTURE`, `FEATURE`, `BUGFIX`, and `INFRA` require `verification_status` and `verification_basis` at commit time. Other categories default to `Unverified`/`ManualInspection` if not provided.

### 4.7 Internal Path Representation

All entity paths are represented as `camino::Utf8PathBuf` internally, consistent with the rest of the ChangeGuard codebase. Conversion to `String` happens only at the SQLite storage boundary. CLI flags accept `String` and convert to `Utf8PathBuf` at the command boundary.

### 4.8 CLI-to-DB Field Mapping

| CLI Flag | DB Column | Notes |
|---|---|---|
| `--description` (on `start`) | `planned_action` | Different names: "description" in CLI is the intent; "planned_action" in DB records what was planned |
| `--summary` (on `commit`) | `summary` | Direct mapping |
| `--reason` (on `commit`) | `reason` | Direct mapping |
| `--entry-type` | `entry_type` | IMPLEMENTATION, ARCHITECTURE, or LESSON |
| `--trace-id` | `trace_id` | Optional correlation ID |
| `--source` | `source` | Auto-set: `CLI` for user-initiated, `WATCHER` for drift-detected |

---

## 5. Config Extensions

### 5.1 New `LedgerConfig` Section

Add to `src/config/model.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LedgerConfig {
    /// Enable tech stack enforcement at transaction start
    #[serde(default)]
    pub enforcement_enabled: bool,

    /// Require verification pass before commit for high-risk categories
    #[serde(default)]
    pub verify_to_commit: bool,

    /// Auto-reconcile watcher drift for the same entity at commit time
    #[serde(default = "default_auto_reconcile")]
    pub auto_reconcile: bool,

    /// Roll back PENDING transactions older than this many hours
    #[serde(default = "default_stale_threshold_hours")]
    pub stale_threshold_hours: u64,

    /// Category-to-stack mappings (defined in config, not just DB)
    #[serde(default)]
    pub category_mappings: Vec<CategoryMapping>,

    /// Watcher patterns for drift detection (supplements hardcoded list)
    #[serde(default)]
    pub watcher_patterns: Vec<WatcherPattern>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryMapping {
    pub ledger_category: String,
    pub stack_category: String,
    pub glob: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatcherPattern {
    pub glob: String,
    pub category: String,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            enforcement_enabled: false,
            verify_to_commit: false,
            auto_reconcile: default_auto_reconcile(),
            stale_threshold_hours: default_stale_threshold_hours(),
            category_mappings: Vec::new(),
            watcher_patterns: Vec::new(),
        }
    }
}
```

### 5.2 Default Config Additions

Add to `src/config/defaults.rs`:

```toml
# [ledger]
# enforcement_enabled = false
# verify_to_commit = false
# auto_reconcile = true
# stale_threshold_hours = 24
#
# [[ledger.watcher_patterns]]
# glob = "**/Cargo.toml"
# category = "INFRA"
#
# [[ledger.category_mappings]]
# ledger_category = "ARCHITECTURE"
# stack_category = "BACKEND_LANG"
```

### 5.3 Config Validation Additions

Add to `src/config/validate.rs`:

- `stale_threshold_hours > 0`
- `watcher_patterns` globs are valid
- `category_mappings` reference known categories and valid stack categories
- `verify_to_commit` warns if no `verify` steps are configured

### 5.4 Runtime Behavior: `verify_to_commit` with No Verify Steps

When `config.ledger.verify_to_commit` is `true` but no verify steps exist (neither `config.verify.steps` nor a manual `-c` command):

- **Config validation** (at load time): emit a warning: `"verify_to_commit is enabled but no [verify] steps are configured. Add steps to config.toml or use 'changeguard verify -c <command>'."`
- **`ledger commit` runtime**: fail with an error: `"Cannot commit: verify_to_commit is enabled but no verification steps are configured. Either add [[verify.steps]] to config.toml or run 'changeguard verify -c <command>' and provide --verification-status manually."`
- **Rationale**: silently allowing unverified commits when verification is required defeats the purpose of the gate. Failing forces the user to either configure verification or explicitly disable the gate.

This does not apply to `ledger note` (which is restricted to low-risk categories that bypass `verify_to_commit`).

---

## 6. CLI Command Structure

### 6.1 New `ledger` Subcommand Group

```
changeguard ledger
  start     --entity PATH --category CAT [--description TEXT] [--issue-ref REF] [--operation-id ID]
  commit    --tx-id UUID --change-type TYPE --summary TEXT --reason TEXT
            [--breaking] [--entry-type TYPE] [--verification-status STATUS] [--verification-basis BASIS]
            [--outcome-notes TEXT] [--auto-reconcile] [--trace-id ID]
  rollback  --tx-id UUID --reason TEXT
  atomic    --entity PATH --category CAT --change-type TYPE --summary TEXT --reason TEXT
            [--breaking] [--entry-type TYPE] [--verification-status STATUS] [--verification-basis BASIS]
            [--auto-reconcile]
  note      --entity PATH --summary TEXT [--reason TEXT]    # Lightweight mode (DOCS|CHORE|TOOLING|REFACTOR only)
  status    [--include-unaudited] [--compact]
  resume                                           # Find most recent PENDING tx
  reconcile [--tx-id UUID] [--entity-pattern GLOB] [--auto-reconcile]
            [--summary TEXT] [--reason TEXT]
  adopt     --tx-id UUID --reason TEXT
  search    QUERY [--category CAT] [--days N] [--breaking-only]
  audit     [--entity PATH] [--include-unaudited]
  stack     [--category CAT]
  register  --rule-type TYPE --payload JSON
  adr       [--output-dir DIR] [--days N]
  diff-deployments --from VERSION --to VERSION
  check-impact --entity PATH
  scaffold  --category CAT --summary TEXT
  artifacts [--tx-id UUID]
```

### 6.2 Existing Command Enhancements

| Command | Enhancement |
|---|---|
| `scan --impact` | After impact report, optionally run `ledger start` for detected entities |
| `impact --ledger-start` | Auto-opens ledger transactions for entities in the impact report |
| `verify` | Results can be auto-attached to a pending transaction via `--tx-id` flag |
| `watch` | Detects drift and creates UNAUDITED transactions |
| `init` | Seeds default tech stack based on auto-detected project type |

---

## 7. Pinned Dependency Additions

New crates required for ledger functionality:

```toml
[dependencies]
# Already present — no version changes
# rusqlite = { version = "0.39.0", features = ["bundled"] }  # FTS5 built-in
# serde = { version = "1.0.228", features = ["derive"] }
# serde_json = "1.0"
# chrono = { version = "0.4.44", features = ["serde"] }
# camino = { version = "1.2.2", features = ["serde1"] }
# thiserror = "2.0"
# miette = { version = "7.6.0", features = ["fancy"] }
# gix = "0.81.0"

# NEW for ledger functionality
uuid = { version = "1.23", features = ["v4", "serde"] }  # Edition 2024 compatible, MSRV 1.85+
```

**Note**: `rusqlite` with `bundled` includes SQLite FTS5 support by default. No additional SQLite compilation flags are needed — the bundled SQLite is compiled with FTS5 enabled.

**Dependency caution**: `uuid` v1.23 is a well-established crate. The `v4` feature generates random UUIDs. We use UUID v4 for transaction IDs because they require no coordination between processes (important for a CLI tool that may run in multiple terminals).

---

## 8. High-Level Delivery Sequence

The implementation should proceed in the following order. Each phase has a verification gate that must pass before the next phase begins.

| Phase | Name | Depends On |
|---|---|---|
| L1 | Transaction Lifecycle & Data Model | Existing ChangeGuard core |
| L2 | Drift Detection & Reconciliation | L1 |
| L3 | Tech Stack Enforcement & Validators | L1 |
| L4 | Search, Audit & ADR Export | L1 |
| L5 | Artifact Reconciliation & Verify Integration | L1, existing verify |
| L6a | Lessons & Organizational Memory | L1 |
| L6b | Cross-Project Federation & Deployments | L1, L6a, existing federated (post-integration) |
| L7 | Polish & Production Readiness | L1–L6a |

---

## Phase L1: Transaction Lifecycle & Data Model [COMPLETED]

**Status**: Implemented in **Track L1-1** (Data Model & Migrations) and **Track L1-2** (Lifecycle & CLI Commands).

### Objective

Establish the transaction data model, lifecycle management, and core CLI commands for recording architectural changes.

### Deliverables

- `src/ledger/types.rs` — [DONE]
- `src/ledger/error.rs` — [DONE]
- `src/ledger/db.rs` — [DONE]
- `src/ledger/transaction.rs` — [DONE]
- `src/ledger/session.rs` — [DONE]
- `src/state/migrations.rs` — [DONE] (M11, M12)
- `src/commands/ledger.rs` — [DONE] (Grouped start/commit/rollback/atomic/note/status/resume)
- `src/config/model.rs` — [DONE] (LedgerConfig)
- `src/cli.rs` — [DONE] (Ledger group)

### Functional Requirements

- `ledger start` opens a PENDING transaction with UUID, category, entity, description
- `ledger commit` transitions PENDING → COMMITTED, requires summary + reason, writes to `ledger_entries` (FTS5 synced automatically via triggers)
- `ledger rollback` transitions PENDING → ROLLED_BACK, requires reason
- `ledger atomic` combines start + commit for single-file surgical edits
- `ledger note` lightweight mode: auto-defaults verification to `ManualInspection`/`Unverified`, restricted to low-risk categories (`DOCS`, `CHORE`, `TOOLING`, `REFACTOR`). High-risk categories (`ARCHITECTURE`, `FEATURE`, `BUGFIX`, `INFRA`) must use the full `start`/`commit` flow. This prevents using `note` to bypass verification requirements.
- `ledger status` shows pending, unaudited, and recent committed transactions
- `ledger status --compact` shows counts only
- `ledger resume` finds the most recent PENDING transaction for the current working directory context
- Transactions with `verification_status` and `verification_basis` fields for non-trivial categories
- Full UUID displayed in all output (never truncate)
- Fuzzy tx_id matching: accept truncated UUID if unique in pending set. If ambiguous, fail deterministically with candidate list (not interactive prompt). Interactive prompting is opt-in via `--interactive` flag. Default behavior is headless-safe (CI/AI compatible).

### Edge Cases

- Starting a transaction for an entity that already has a PENDING transaction → conflict error
- Committing with mismatched verification requirements for the category
- Committing a transaction that was started in a different session → warn but allow
- Ghost commit guard: if change_type is not DELETE and entity doesn't exist on disk → warn, annotate as `MODIFIED (MISSING)`
- UUID collision (astronomically unlikely but handle gracefully)
- Empty entity string → reject
- Very long summary/reason strings → no truncation in storage, truncate in display

### Path Normalization

All entity paths are normalized before storage:

1. Resolve relative to workspace root (git repo root)
2. Strip UNC prefix (`\\?\`) on Windows long paths
3. Convert backslashes to forward slashes
4. Lowercase drive letters on Windows
5. Strip leading `./`
6. **Case-folding is conditional on filesystem semantics**:
   - On case-insensitive filesystems (Windows NTFS, macOS APFS default): apply Unicode-aware lowercase (`.to_lowercase()`) for `entity_normalized` — enables conflict detection for `Foo.rs` vs `foo.rs`.
   - On case-sensitive filesystems (Linux ext4, WSL2 ext4, macOS APFS case-sensitive): do NOT case-fold `entity_normalized` — `Foo.rs` and `foo.rs` are distinct files and must remain distinct in conflict detection.
   - Detection: at `init` time, probe the filesystem by creating a temp file with mixed case and checking if a lowercase-open finds it. Cache the result in `.changeguard/state/` so it doesn't need re-probing every run.
7. Store as `entity_normalized` (conditionally case-folded) for conflict detection
8. Store as `entity` (original casing from filesystem) for display

### Acceptance Criteria

- `ledger start --entity src/main.rs --category FEATURE --description "Add ledger support"` creates a PENDING transaction
- `ledger commit --tx-id <uuid> --change-type MODIFY --summary "Added ledger module" --reason "Incorporate Project Ledger"` commits the transaction
- `ledger status` shows the transaction in the correct lifecycle state
- `ledger atomic` completes in a single operation
- `ledger note` does not require verification fields; restricted to `DOCS`, `CHORE`, `TOOLING`, `REFACTOR` categories
- `ledger resume` returns the most recent PENDING transaction
- Truncated UUID is accepted if unique
- Re-running `init` is safe (idempotent)
- Full UUID is always displayed in output

### Verification Gate

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test` including new `tests/ledger_lifecycle.rs`
- Test: start → commit round trip
- Test: start → rollback round trip
- Test: atomic round trip
- Test: note mode skips verification requirements
- Test: resume finds pending transaction
- Test: fuzzy tx_id matching
- Test: conflict detection on duplicate PENDING entity
- Test: ghost commit guard

---

## Phase L2: Drift Detection & Reconciliation [IN PROGRESS]

**Status**: Implementing in **Track L2-1** (Ledger Drift Detection) and **Track L2-2** (Reconciliation & Adoption).

### Objective

Connect ChangeGuard's existing file watcher to the transaction system so that untracked filesystem changes produce UNAUDITED transactions that must be reconciled.

### Deliverables

- `src/ledger/drift.rs` — Watcher event → UNAUDITED transaction bridge [DONE]
- `src/commands/ledger_reconcile.rs` — `ledger reconcile` command [IN PROGRESS]
- `src/commands/ledger_adopt.rs` — `ledger adopt` command [IN PROGRESS]
- Extend `src/watch/` — Emit drift events to transaction system [DONE]
- Extend `src/commands/ledger_status.rs` — Separate active session from stale drift [DONE]
- `src/ledger/session.rs` — Stale transaction cleanup (background, configurable threshold) [TODO]

### Functional Requirements

- When the file watcher detects a change to a watched file and no matching PENDING transaction exists, create an UNAUDITED transaction with `source: WATCHER`
- Deduplicate drift: if the same entity generates multiple UNAUDITED entries in the same session, show as one entry with count instead of N UUIDs
- `ledger reconcile` links an UNAUDITED transaction to a documented reason and transitions it to RECONCILED (preserves `source: WATCHER` provenance, unlike `commit` which transitions to COMMITTED with `source: CLI`)
  - Supports `--tx-id` for specific transaction
  - Supports `--entity-pattern` glob for bulk reconciliation
  - Supports `--auto-reconcile` flag (from config or CLI)
- `ledger adopt` converts an UNAUDITED transaction back to PENDING for recovery
- `ledger status` separates "active session" (pending from current process) from "stale drift" (orphaned UNAUDITED entries)
- Stale transaction handling: PENDING transactions older than `stale_threshold_hours` (default 24) are flagged as stale in `ledger status` with a prominent warning, but are NOT auto-rolled-back. Auto-mutating state during a read-like command (`status`, `start`) is too aggressive — long-running legitimate changes, reboots, or multi-terminal work can all look "stale." Instead, `ledger status` shows stale transactions in a separate section and suggests `ledger rollback --tx-id <id>` or `ledger adopt --tx-id <id>` to clean up. A `--prune-stale` flag on `ledger status` performs the rollback explicitly when the user confirms.

### High-Signal File List

Hardcoded files that always trigger drift detection regardless of config:

```
.env, .env.*, Cargo.toml, package.json, tsconfig.json, docker-compose*.yml,
Dockerfile*, *.prisma, pyproject.toml, go.mod, requirements.txt, Makefile,
Justfile, build.gradle, pom.xml, .github/workflows/*.yml
```

### Hardcoded Ignore List

Never trigger drift detection:

```
target/, node_modules/, .git/, .changeguard/, *.lock, *.log, .DS_Store,
dist/, build/, coverage/, __pycache__/, .idea/, .vscode/, *.swp, *.swo
```

### Edge Cases

- Watcher fires during an active PENDING transaction for the same entity → do NOT create UNAUDITED (the change is expected)
- Multiple rapid saves to the same file → debounce into one UNAUDITED entry (reuse existing watcher debounce)
- Entity deleted after UNAUDITED creation → still reconcile, annotate as deleted
- Reconcile with `--entity-pattern` matching many entries → bulk commit, warn on count
- Auto-reconcile at commit time: when committing a transaction for entity X, also reconcile any UNAUDITED entries for the same entity_normalized

### Auto-Reconcile Semantics

The `--auto-reconcile` flag appears in three contexts. Their interaction is fully specified here:

1. **Config default** (`config.ledger.auto_reconcile`, default `true`): Sets the baseline behavior. When true, both `ledger commit` and `ledger reconcile` auto-reconcile matching UNAUDITED entries unless explicitly disabled.

2. **`ledger commit --auto-reconcile`** (CLI flag): Overrides the config default for this commit operation only. `--auto-reconcile` forces it on even if config says false. `--no-auto-reconcile` (or omitting the flag when config default is true) follows the config. At commit time, any UNAUDITED entries for the same `entity_normalized` are transitioned to RECONCILED.

3. **`ledger reconcile --auto-reconcile`** (CLI flag): Overrides the config default for this reconcile operation only. Same override semantics as commit. When active, all matching UNAUDITED entries are transitioned to RECONCILED without requiring individual `--tx-id` or `--reason` per entry (the bulk reason comes from the CLI `--reason` flag or defaults to "Auto-reconciled via bulk operation").

Precedence: CLI flag > config default. If neither is specified, the config default applies.

### Acceptance Criteria

- Modifying a watched file without a PENDING transaction creates an UNAUDITED entry
- `ledger reconcile --tx-id <uuid>` successfully commits the UNAUDITED entry
- `ledger reconcile --entity-pattern "src/**/*.rs"` bulk-reconciles matching entries
- `ledger status --compact` shows counts by section
- Same entity modified multiple times in one session shows as one deduplicated entry
- Auto-reconcile works when committing a transaction for a drifted entity
- Stale PENDING transactions are cleaned up per configured threshold

### Verification Gate

- `cargo test` including new `tests/ledger_drift.rs`
- Test: UNAUDITED creation on untracked change
- Test: reconciliation round trip
- Test: bulk reconciliation by pattern
- Test: deduplication of same-entity drift
- Test: auto-reconcile integration
- Test: stale transaction cleanup

---

## Phase L3: Tech Stack Enforcement & Validators [IN PROGRESS]

**Status**: Implementing in **Track L3-1** (Enforcement Data Model & Registration) [DONE] and **Track L3-2** (Enforcement & Validation Logic) [IN PROGRESS].

### Objective

Implement architectural constraint enforcement that prevents violations before code is written and validates at commit time.

### Deliverables

- `src/ledger/enforcement.rs` — Tech stack rule checking at transaction start [IN PROGRESS]
- `src/ledger/validators.rs` — Shell-command validators at commit time [IN PROGRESS]
- `src/commands/ledger_stack.rs` — `ledger stack` command (read rules) [DONE]
- `src/commands/ledger_register.rs` — `ledger register` command (add rules/validators) [DONE]
- Extend `src/commands/init.rs` — Auto-detect tech stack, seed defaults [TODO]
- `src/state/migrations.rs` — New migration for tech_stack, commit_validators, category_stack_mappings tables [DONE] (M13)

### Functional Requirements

**Tech Stack Rules**:

- `ledger register --rule-type TECH_STACK --payload '{"category":"DATABASE","name":"SQLite","rules":["NO JSONB columns","NO stored procedures"],"locked":true}'`
- At `ledger start`, map the transaction's category to applicable stack categories via `category_stack_mappings`
- Check if the description matches any `NO <term>` pattern in applicable rules
- If violation found, REJECT the transaction with the rule and suggestion
- Locked entries cannot be overwritten without explicit `--force` flag
- `ledger stack` displays all registered tech stack entries with their rules

> **Design caveat — heuristic enforcement.** The `NO <term>` pattern matching operates on the transaction's `description` field — a free-text string provided at `ledger start`. This means enforcement is a coarse gate: it catches obvious violations (e.g., a description containing "JSONB column" when the rule says `NO JSONB columns`), but it will not catch violations that require code-level analysis. Agents should be encouraged to include implementation-relevant keywords in descriptions, but this remains a heuristic, not a guarantee. A future code-aware enforcement pass (e.g., scanning the staged diff for prohibited patterns) could strengthen this, but is out of scope for L3.

**Commit Validators**:

- `ledger register --rule-type VALIDATOR --payload '{"category":"FEATURE","name":"type-check","executable":"cargo","args":["check","{entity}"],"timeout_ms":30000,"validation_level":"ERROR"}'`
- At `ledger commit`, run all validators matching the transaction's category
- Validators receive the entity path (absolute) by substituting `{entity}` in any element of the `args` array (not shell expansion)
- ERROR-level validators that exit non-zero BLOCK the commit
- WARNING-level validators that exit non-zero produce warnings but do not block
- Validators run with the same process policy as ChangeGuard's verify command (timeout, output capture)
- When `config.ledger.verify_to_commit` is true, also require the ChangeGuard `verify` command to pass before allowing commit

**Auto-Detection at Init**:

- `changeguard init` detects project type from file markers:
  - `Cargo.toml` → Rust (register `BACKEND_LANG: Rust`, seed `cargo check` validator)
  - `package.json` → Node/TS (register `BACKEND_LANG: TypeScript`, seed `npx tsc --noEmit` validator)
  - `pyproject.toml` or `requirements.txt` → Python (register `BACKEND_LANG: Python`)
  - `.env*` → register `ENV_VAR` watcher pattern
  - `docker-compose*.yml` → register `INFRA_CONFIG` watcher pattern

### Edge Cases

- Empty tech stack (no rules registered) → enforcement is a no-op, all transactions allowed
- Validator command not found → WARNING, not error (don't block commits on missing tools)
- Validator timeout → treat as WARNING (not block, since timeout may be env-specific). Note: this differs from the verify runner where timeouts are hard errors; commit validators are softer because they run at commit time where the user's work is at stake.
- Circular category mappings → reject at registration time
- `--force` on locked tech stack → warn prominently, require confirmation
- Multiple validators for same category → run all, accumulate results

### Acceptance Criteria

- Registering a tech stack rule prevents transactions that violate it
- Commit validators run and block/allow commits appropriately
- `ledger stack` displays registered rules clearly
- Auto-detection at init seeds appropriate defaults for Rust/Node/Python projects
- Empty tech stack = no enforcement (opt-in)
- Locked rules require `--force` to override

### Verification Gate

- `cargo test` including new `tests/ledger_enforcement.rs`
- Test: tech stack rule blocks violating transaction
- Test: commit validator blocks on ERROR exit
- Test: commit validator warns on WARNING exit
- Test: auto-detection seeds correct tech stack
- Test: locked rule enforcement

---

## Phase L4: Search, Audit & ADR Export [COMPLETED]

**Status**: Implemented in **Track L4-1** (ADR Generation) and **Track L4-2** (FTS5 Search).

### Objective

Make the audit trail queryable and exportable through FTS5 search, holistic project auditing, and Architectural Decision Record generation.

### Deliverables

- `src/ledger/adr.rs` — MADR-format ADR exporter [DONE]
- `src/commands/ledger_adr.rs` — `ledger adr` command [DONE]
- `src/commands/ledger_search.rs` — `ledger search` command (positional query) [DONE]
- `src/ledger/db.rs` — FTS5 search query logic [DONE]
- `src/commands/ledger_audit.rs` — `ledger audit` command (holistic project state) [TODO]
- `src/commands/ledger_scaffold.rs` — `ledger scaffold` command [TODO]

### Functional Requirements

**FTS5 Search**:

- `ledger search "auth logic"` — full-text search over entity, summary, reason fields
- `ledger search --category FEATURE "pagination"` — filter by category
- `ledger search --days 30 "breaking"` — limit to recent entries
- `ledger search --breaking-only` — only entries with `is_breaking = 1`
- Positional query argument (not `--query` flag) per feedback
- Results ranked by FTS5 relevance, deterministically second-sorted by committed_at (descending)

**Project Audit**:

- `ledger audit` — holistic view combining:
  - Pending transactions (active session vs stale)
  - Unaudited drift count
  - Untracked drift (files changed but no UNAUDITED entry yet — uses git status)
  - Recent breaking changes (last 7 days)
  - Verification coverage (what % of commits are verified)
  - Tech stack compliance status
- `ledger audit --entity src/main.rs` — audit history for a specific entity
- `ledger audit --include-unaudited` — include UNAUDITED in the audit view

**ADR Export**:

- `ledger adr` — export entries with `entry_type = ARCHITECTURE` or `is_breaking = 1` as MADR-format markdown
- `ledger adr --output-dir docs/adr` — write ADR files to specified directory
- `ledger adr --days 30` — limit to recent entries
- MADR template structure:
  ```markdown
  # {N}. {summary}

  - **Status**: {change_type}
  - **Category**: {category}
  - **Breaking**: {is_breaking}

  ## Context
  {reason}

  ## Decision
  {summary}

  ## Consequences
  {is_breaking description if applicable}
  ```

**Scaffold**:

- `ledger scaffold --category FEATURE --summary "Add auth"` — generate a TOML template for a ledger entry, printed to stdout for review before committing

### Edge Cases

- Empty search results → clear "no results" message, suggest broader query
- FTS5 special characters in query → escape or reject with clear message
- ADR export with no qualifying entries → inform user, don't create empty files
- Very large audit output → paginate or truncate with summary counts
- Scaffold for unknown category → reject

### Acceptance Criteria

- FTS5 search returns relevant results ranked by relevance
- Category and date filters work correctly
- Positional search query works
- `ledger audit` provides comprehensive project state overview
- ADR export generates valid MADR markdown files
- Scaffold generates correct TOML template

### Verification Gate

- `cargo test` including new `tests/ledger_search.rs`
- Test: FTS5 search round trip (insert → search → verify results)
- Test: category filter
- Test: date filter
- Test: ADR output format
- Test: audit output structure

---

## Phase L5: Artifact Reconciliation & Verification Integration

### Objective

Bridge ChangeGuard's verification system and impact analysis with the ledger transaction system, and provide git-diff-based artifact suggestions.

### Deliverables

- `src/ledger/reconcile.rs` — Artifact reconciliation via git diff
- `src/commands/ledger_artifacts.rs` — `ledger artifacts` command
- Extend `src/commands/verify.rs` — `--tx-id` flag to attach results to transaction
- Extend `src/commands/impact.rs` — `--ledger-start` flag to auto-open transactions
- Extend `src/commands/scan.rs` — bridge from scan to ledger start
- `src/impact/packet.rs` — Add optional `tx_id` field to ImpactPacket

### Functional Requirements

**Artifact Reconciliation**:

- `ledger artifacts` runs `git diff --name-status` and `git diff --cached --name-status` and `git ls-files --others` to suggest created, modified, and deleted files
- `ledger artifacts --tx-id <uuid>` suggests artifacts for a specific transaction
- Auto-classifies test files (paths containing `/test/`, `.test.`, `.spec.`) and marks them separately
- Output: table of (Status, Path, Classification) — e.g., `M src/main.rs source`, `A tests/foo.rs test`

**Verification Integration**:

- `verify --tx-id <uuid>` runs verification and auto-sets `verification_status` and `verification_basis` on the transaction based on results:
  - All pass → `Verified` / `Tests` (or `Lint` if only lint steps)
  - Any fail → `Failed` / appropriate basis
- When `config.ledger.verify_to_commit` is true, `ledger commit` refuses commits that don't have `verification_status: Verified`
- Verification results are linked to the transaction via `snapshot_id`

**Impact-to-Ledger Bridge**:

- `impact --ledger-start` auto-opens PENDING transactions for all entities in the impact report, grouped under a shared `operation_id`
  - Category defaults to `ARCHITECTURE` for protected paths, `FEATURE` otherwise
  - Description auto-populated from impact analysis: "ChangeGuard impact: {risk_level} risk, {n} files affected"
  - `operation_id` is auto-generated (UUID) so the group is visible in `ledger status`
- `scan --impact --ledger-start` combines scan + impact + ledger start in one flow

### Edge Cases

- `ledger artifacts` with no git changes → "clean working tree" message
- `verify --tx-id` with non-existent or non-PENDING transaction → error
- `verify --tx-id` with already-COMMITTED transaction → error (immutable)
- `impact --ledger-start` with 50+ changed files → warn about transaction volume, ask for confirmation
- Multiple transactions for same entity (from impact) → skip, warn about existing PENDING

### Acceptance Criteria

- `ledger artifacts` correctly identifies and classifies changed files
- `verify --tx-id` attaches verification results to the transaction
- `verify_to_commit` config prevents unverified commits for high-risk categories
- `impact --ledger-start` creates transactions for impact-detected entities
- The full workflow works: scan → impact → ledger start → edit → verify → ledger commit

### Verification Gate

- `cargo test` including new `tests/ledger_artifacts.rs`
- Test: artifact reconciliation from git diff
- Test: verify → commit integration
- Test: impact → ledger start integration
- Test: verify_to_commit gate
- Test: full workflow round trip

---

## Phase L6a: Lessons & Organizational Memory

### Objective

Add lesson/convention storage for organizational memory — a local analog of Open Brain that surfaces relevant past decisions when starting new work.

### Deliverables

- `src/ledger/lesson.rs` — Lesson/convention storage and retrieval
- `src/commands/lesson_add.rs` — `lesson add` command
- `src/commands/lesson_search.rs` — `lesson search` command
- `src/state/migrations.rs` — `lessons` table, `lessons_fts` virtual table, FTS5 triggers

### Functional Requirements

**Lessons (Open Brain Analog)**:

- `changeguard lesson add --category CONVENTION --issue "Always use scoped threads" --action "Use std::scoped threads, not std::thread::spawn for bounded work"`
- `changeguard lesson add --category GRAVEYARD --issue "Tried macro-based DI, too complex" --action "Use trait objects for runtime DI instead"`
- `changeguard lesson search "thread"` — full-text search over lessons
- Lessons are stored in a `lessons` table with FTS5 index
- At `ledger start`, automatically search lessons for the entity and surface relevant conventions/warnings
- At `ledger commit` with `--globalize` or `is_breaking`, auto-log as a GRAVEYARD lesson
- Lesson fields: category (GRAVEYARD, ENVIRONMENT, CONVENTION), issue_context, corrective_action, confidence (high/medium/low), scope (GLOBAL/PROJECT_BOUND), project_scope

### Edge Cases

- Lesson search returns too many results → limit to 10, suggest narrower query
- Duplicate lesson on same topic → warn, allow with `--force`

### Acceptance Criteria

- Lesson add/search round trip works
- Lessons are surfaced at `ledger start` time when relevant
- Breaking changes auto-globalize as lessons when `--globalize` flag is used

### Verification Gate

- `cargo test` including new `tests/ledger_lessons.rs`
- Test: lesson add + search
- Test: lesson surfacing at transaction start
- Test: auto-globalize on breaking change commit

---

## Phase L6b: Cross-Project Federation & Deployments

> **Post-integration phase.** L6b significantly expands the existing ChangeGuard federation model (which reads sibling `schema.json` files and never writes to siblings). This phase adds sibling ledger syncing, service dependency mapping, deployment tracking, and cross-project impact checks — all of which are meaningful capabilities but are not part of the first ledger merge. Implement L1–L6a first, validate end-to-end, then tackle L6b as a separate roadmap item.

### Objective

Extend the existing ChangeGuard federated system with ledger-level cross-project awareness and deployment boundary tracking.

### Deliverables

- Extend `src/federated/` — Sync ledger entries from sibling repos
- Extend `src/commands/federate.rs` — `federate sync-ledger` subcommand
- `src/commands/ledger_diff_deployments.rs` — `ledger diff-deployments` command
- `src/commands/ledger_check_impact.rs` — `ledger check-impact` command
- `src/state/migrations.rs` — service_dependencies, deployments tables
- Extend `src/state/storage.rs` — Federated ledger query support

### Functional Requirements

**Cross-Project Federation**:

- `federate sync-ledger` pulls recent ledger entries from known sibling repos
- Store synced entries in `ledger_entries` with `origin: SIBLING` annotation
- `ledger search` with `--include-federated` includes sibling entries in results
- `ledger audit` shows cross-project dependencies and recent sibling breaking changes
- `service_dependencies` table maps which projects depend on which

**Deployment Boundaries**:

- `ledger register --rule-type DEPENDENCY --payload '{"service_name":"api","depends_on":"shared-lib","dependency_type":"SHARED_LIB"}'`
- Deployment tracking via `deployments` table
- `ledger diff-deployments --from v1.2 --to v1.3` shows all ledger entries between two deployment boundaries, with breaking change stats

**Cross-Project Impact**:

- `ledger check-impact --entity src/api.rs` queries `service_dependencies` for downstream services and checks recent breaking changes in sibling repos

### Edge Cases

- Sibling repo offline or inaccessible → warn, continue with local data
- Circular service dependencies → detect and warn at registration
- Stale federated data → show last sync timestamp, suggest re-sync

### Acceptance Criteria

- Federated sync pulls and stores sibling ledger entries
- `ledger search --include-federated` includes cross-project results
- Service dependency mapping works

### Verification Gate

- `cargo test` with mock federated data
- Test: federated sync with mock sibling
- Test: service dependency registration

---

## Phase L7: Polish & Production Readiness

### Objective

Polish the user experience, add convenience features, and ensure the full system is ready for daily use.

### Deliverables

- Enhanced `ledger status` with color-coded sections
- `ledger status --compact` with counts only
- Transaction display improvements (relative timestamps, category icons)
- Help text and examples for all commands
- Shell completion scripts (bash, zsh, PowerShell)
- TOML entry export (optional, nice-to-have)
- Documentation updates (skill.md, README)

### Functional Requirements

- All ledger commands produce clear, actionable output
- Error messages explain what failed, why, and what to do next
- `ledger status` uses color to distinguish pending (yellow), committed (green), unaudited (red)
- Relative timestamps in status ("2 hours ago") alongside absolute
- Help examples for common workflows:
  - `changeguard ledger atomic --entity src/main.rs --category FEATURE --change-type MODIFY --summary "Add X" --reason "Needed for Y"`
  - `changeguard ledger note --entity docs/api.md --summary "Update endpoint docs"`
  - `changeguard impact --ledger-start && changeguard verify --tx-id <uuid>`

### Acceptance Criteria

- A new user can understand and use all ledger commands without guessing
- Help text covers the most common workflows
- Error messages are actionable
- All commands have consistent output formatting

### Verification Gate

- Full test suite green
- Manual smoke test of all commands
- Help text review

---

## 9. Milestones

### Milestone L-A — Transaction Foundation

Complete:

- Phase L1 (Transaction Lifecycle & Data Model)

**Deliverable**: Users can start, commit, rollback, and query transactions. The audit trail exists.

### Milestone L-B — Drift & Enforcement

Complete:

- Phase L2 (Drift Detection & Reconciliation)
- Phase L3 (Tech Stack Enforcement & Validators)

**Deliverable**: The system independently detects drift, enforces tech stack rules, and validates commits. Trust-and-verify model is operational.

### Milestone L-C — Search & Integration

Complete:

- Phase L4 (Search, Audit & ADR Export)
- Phase L5 (Artifact Reconciliation & Verify Integration)

**Deliverable**: The audit trail is queryable, ADR export works, and ChangeGuard's verification is linked to the ledger. The full workflow (scan → start → edit → impact → verify → commit) works end to end.

### Milestone L-D — Federation & Organizational Memory

Complete:

- Phase L6a (Lessons & Organizational Memory), L6b (Cross-Project Federation & Deployments)

**Deliverable**: Cross-project awareness and organizational memory (lessons) are functional.

### Milestone L-E — Production Ready

Complete:

- Phase L7 (Polish & Production Readiness)

**Deliverable**: The complete system is documented, polished, and ready for daily use across projects.

---

## 10. Testing Strategy

### Unit Tests

Use for:

- Transaction lifecycle state transitions
- Category/change type/verification enums
- Path normalization
- Tech stack rule matching
- Commit validator execution
- FTS5 search ranking
- ADR template generation
- Lesson storage and retrieval
- Config validation

### Integration Tests

Use for:

- Full transaction round trips (start → commit, start → rollback)
- Drift detection (watcher → UNAUDITED → reconcile)
- Artifact reconciliation (git diff → entity suggestions)
- Verify → commit integration
- Impact → ledger start integration
- Federated sync with mock sibling repos

### Manual Validation

Always validate on:

- Windows 11 + PowerShell
- WSL2 Ubuntu
- Ubuntu native if available

### CI Requirements

Run at minimum:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test`
- `cargo audit`
- `cargo deny check`

---

## 11. AI Implementation Protocol

Each AI implementation pass should follow this discipline:

1. Implement one phase or tightly bounded subphase only.
2. Add or update tests in the same pass.
3. Run format, lint, and test gates.
4. Document deviations explicitly.
5. Do not silently introduce architecture from future phases.
6. Prefer partial working behavior over speculative cleverness.
7. Never use `unwrap`/`expect` in production code paths.
8. All outputs must be deterministically ordered (stable sort).
9. Error messages must explain what, why, and next step.

---

## 12. Migration Strategy

### 12.1 Existing State Compatibility

All existing ChangeGuard tables remain untouched. New tables are added via incremental migrations in `src/state/migrations.rs`. The migration version counter continues from the current latest.

### 12.2 Migration Ordering

The existing migrations.rs contains 10 entries (M1–M10). New migrations start at M11.

```
M11: CREATE TABLE transactions (...), indexes
M12: CREATE TABLE ledger_entries (...), CREATE VIRTUAL TABLE ledger_fts (...), FTS5 triggers
M13: CREATE TABLE tech_stack, commit_validators, category_stack_mappings, watcher_patterns
M14: CREATE TABLE service_dependencies, deployments
M15: CREATE TABLE lessons, CREATE VIRTUAL TABLE lessons_fts (...), FTS5 triggers
```

### 12.3 SQLite WAL Mode

The `StorageManager` must enable WAL journal mode on connection for concurrent CLI process support:

```rust
conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")?;
```

WAL mode allows concurrent reads from multiple terminal processes while serializing writes. The `busy_timeout` of 5 seconds prevents immediate `SQLITE_BUSY` errors when two CLI instances write simultaneously.

### 12.4 Watcher Pattern Persistence

Watcher patterns come from three sources, with precedence:

1. **Hardcoded high-signal list** (compiled into the binary, cannot be removed)
2. **Config-defined patterns** (`config.ledger.watcher_patterns` in `config.toml`, version-controllable)
3. **Runtime-registered patterns** (`ledger register --rule-type WATCHER`, stored in `watcher_patterns` DB table with `source = 'DB'`)

At watcher startup, all three sources are merged. Config patterns take precedence over DB patterns (same glob = config wins). Hardcoded patterns always apply.

Each migration is a separate `M` constant in the migrations array, applied atomically by `rusqlite_migration`.

### 12.5 No Down-Migrations

Following ChangeGuard's existing pattern, there are no down-migrations. The database only grows.

---

## 13. Security Considerations

1. **Entity path confinement**: All entity paths must resolve within the git repo root. Reject absolute paths outside the workspace.
2. **Validator command injection**: The `{entity}` placeholder is substituted directly into argv elements, not through shell expansion. The validator execution model uses `Command::new(executable).args(args)` (same as existing verify runner's `PreparedStep`), which avoids shell injection entirely.
3. **Commit validator isolation**: Validators run with the same ProcessPolicy as the verify command (blocked dangerous commands, timeout enforcement).
4. **No secret storage**: Never store API keys, tokens, or credentials in ledger entries. The existing secret redaction system applies.
5. **UUID predictability**: UUID v4 provides sufficient randomness for transaction IDs. Do not use sequential IDs for transactions.

---

## 14. Performance Considerations

1. **FTS5 overhead**: The FTS5 virtual table adds minimal write overhead (~10% on insert). Read performance is excellent for the expected data volumes (thousands of entries, not millions).
2. **Validator execution**: Commit validators are the highest latency operation. Run them in parallel where possible. Total commit time = max(validator) not sum(validator).
3. **Stale transaction detection**: Run lazily (on next `status` or `start` call) rather than as a background thread. Stale transactions are flagged, not auto-rolled-back.
4. **Drift deduplication**: Deduplicate in memory before writing to DB. The watcher debounce window already collapses rapid events.
5. **Search result limits**: Default limit of 20 results for `ledger search`. Allow `--limit` override.

---

## 15. Final Implementation Warning

The most likely way to fail this incorporation is to overbuild the governance ceremony before the core transaction lifecycle is stable.

The most important success criteria for each phase are:

- Transactions can be started, committed, and queried
- Drift is detected and reconciled
- Enforcement blocks real violations
- Search returns useful results
- The full workflow works end to end

**Reliability comes first. Sophistication comes second.**
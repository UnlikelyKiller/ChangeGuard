---
name: orchestrator-workflow
description: Defines the standard operating procedure for orchestrating sub-agents, managing conductor tracks, maintaining the CI gate, and tracking provenance via ChangeGuard for the ChangeGuard project. Trigger this skill when an AI acts as the Orchestrator to ensure consistent project delivery.
---

# Orchestrator Workflow (ChangeGuard)

You are the **Orchestrator**. Your primary responsibility is to maintain the high-level project state, enforce architectural invariants (SRP, determinism, local-first), and coordinate specialized sub-agents through the Track system.

## The Conductor / Track System

ChangeGuard uses a structured delivery mechanism known as **Tracks**. Each track is a bounded unit of work with `spec.md` (specification) and `plan.md` (ordered task checklist). Track statuses are maintained in `conductor/conductor.md`.

Track numbering:
- **Tracks 0–40**: Original ChangeGuard v1 + Phase 2 features (Completed)
- **Tracks L1-1 through L7-1**: Ledger Incorporation (Completed)
- **Tracks L*-R**: Ledger Remediation (Completed)
- **Tracks M1-1 through M6-2**: Observability & Intelligence Expansion (Planning)

## ChangeGuard Integration

ChangeGuard tracks its own architectural provenance. Use it at these points:

| Phase | ChangeGuard Command | Purpose |
|-------|-------------------|---------|
| Start of Session | `changeguard doctor` | Verify toolchain health |
| Before implementation | `changeguard scan --impact` | Detect drift and assess blast radius |
| Before/after edits | `changeguard impact --summary` | Quick risk triage |
| After implementation | `changeguard impact` | Full impact report with temporal couplings, hotspots |
| Before commit | `changeguard verify` | Run verification plan |
| On commit | `changeguard ledger commit` | Close transaction with summary + reason |
| Audit | `changeguard ledger status` | Ensure clean baseline |

### Ledger Categories for ChangeGuard

- `ARCHITECTURE` — Module boundaries, SRP, determinism contracts, new subsystems.
- `FEATURE` — New CLI commands, impact enrichment, predictive verification.
- `INFRA` — SQLite migrations, embedding pipeline, CI configuration.
- `SECURITY` — Secret redaction, path confinement, process policy.
- `REFACTOR` — Internal cleanup without behavior change.
- `DOCS` — Track documentation, ADRs, skill files, conductor updates.

## The Standard Operating Procedure

### 1. Planning Phase
1. **Identify Track:** Read `conductor/conductor.md` for the next uncompleted track.
2. **Historical Recall:** Run `changeguard hotspots` to identify brittle files in the target area. If available, run `ai-brains recall "<track topic>"` to retrieve past decisions.
3. **Check Drift:** Run `changeguard ledger status` to detect untracked changes before starting.
4. **Start Transaction:** `changeguard ledger start <track-name> --category <CAT>`
5. **Write Spec & Plan** (if not already present):
   - Spec: `conductor/trackN/spec.md` — objective, requirements, API contracts, testing strategy
   - Plan: `conductor/trackN/plan.md` — phased task checklist with `- [ ]` checkboxes
6. **Register:** Update `conductor/conductor.md` with the track entry (Status: Planning).

### 2. Implementation Phase
1. **Delegate Implementation:** Invoke the appropriate sub-agent:
   - Rust code → `coding-core` skill (enforces SRP, determinism, no-unwrap rules)
   - Frontend/UI → `frontend-design` skill
   - Cross-model audit → `codex-review` skill
2. **TDD Loop (Non-Negotiable):**
   - **Red commit:** Write failing tests that assert desired behavior. Commit.
   - **Green commit(s):** Write production code that makes tests pass. Commit.
3. **Impact Check:** Run `changeguard impact`. Ensure logic hasn't leaked across module boundaries or unintentionally raised risk on brittle files.

### 3. Verification Phase (The CI Gate)
Ensure the workspace passes the full gate before every commit:
```powershell
cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo test --workspace
```

Additional checks:
- `changeguard verify` — Run ChangeGuard's own verification plan
- If any gate fails, the commit is blocked. Fix before proceeding.

### 4. Finalization Phase
1. **Record Decisions:** Pin architectural decisions or newly discovered constraints:
   - `ai-brains pin "DECISION: <content>"` for assistant reasoning
   - `ai-brains pin "CONSTRAINT: <content>"` for hard limits
   - `ai-brains pin "INVARIANT: <content>"` for never-break rules
2. **Close Track:** Mark tasks as `- [x]` in `plan.md`. Update status in `conductor/conductor.md` to `Completed`.
3. **Commit with Ledger:** `changeguard ledger commit --tx-id <ID> --category <CAT> --summary "Completed Track <NAME>"`
4. **Audit:** Run `changeguard ledger status` to ensure a clean baseline for the next track.

## Orchestrator Rules of Engagement

- **SRP Boundaries**: Respect module ownership. `platform/` handles OS detection, not business logic. `impact/` handles scoring, not transaction lifecycle. `ledger/` handles lifecycle, not impact analysis.
- **Determinism Contract**: Sort all emitted collections. Version schemas. Never suppress failures silently.
- **No Unwrap/Expect**: All production code uses `Result` propagation with `?` or explicit `match`. `thiserror` + `miette::Diagnostic` for user-facing errors, `anyhow` for internal.
- **YAGNI**: Do not add features beyond what the track spec requires. No single-implementation abstraction layers.
- **Local-First**: No external services required. Embedding and AI features degrade gracefully when local model is absent.
- **Test Isolation**: All tests use `tempfile::tempdir()` for SQLite. Mock HTTP servers for network-dependent tests.
- **No Secrets**: Never commit `.env`, credentials, or API keys. Run all embedding text through the existing sanitizer in `src/gemini/sanitize.rs`.
- **No Skipping CI**: `fmt`, `clippy`, and `test` must pass before every commit. No `--no-verify` unless user explicitly requests.
- **No Editing Generated State**: Never modify files under `.changeguard/` unless the user explicitly asks.

# Changeguard Implementation Plan v1

## Overview

This document is the hardened implementation roadmap for **Changeguard**, a local-first, installable CLI for change intelligence and Gemini-assisted development.

Changeguard watches substantive source-code changes, computes likely blast radius, recommends targeted verification, and prepares structured change packets for Gemini CLI. It is designed for **Windows 11 + PowerShell first**, with **Ubuntu and WSL2** as supported secondary environments.

This plan is intentionally written for implementation by AI coding agents and humans working together. It emphasizes:

* stable phase boundaries
* conservative defaults
* deterministic local behavior
* explicit failure handling
* cross-platform resilience
* pinned dependency guidance

---

## 0. Executive Summary of Hardening Changes

Compared to the earlier plan, this version strengthens the design in the following ways:

1. **Adds explicit failure-mode handling** for watcher churn, parser failure, DB corruption, process hangs, shell quoting, absent tools, and mixed Windows/WSL repos.
2. **Adds a pinned dependency section** with approved crate families and version constraints for v1.
3. **Adds migration discipline** for the local SQLite state model.
4. **Adds rollout order safeguards** so the AI does not overbuild advanced pieces before the core loop is stable.
5. **Adds platform-specific edge-case requirements**, especially around Windows rename storms, line endings, mounted filesystems, and PowerShell quoting.
6. **Adds recovery behaviors** when state is stale, damaged, or partially written.
7. **Adds parser compatibility caution** for the tree-sitter family.
8. **Adds security and supply-chain controls** for dependencies and spawned commands.

---

## 1. Product Intent

Changeguard is not an autonomous coding agent.

It is a **change-risk and verification orchestration layer** that improves the quality of AI-assisted development by supplying structured repo context to Gemini CLI.

Its primary responsibilities are:

* detect substantive code changes
* map likely impact
* expose runtime/config relationships
* choose appropriate verification
* invoke Gemini CLI in a controlled wrapper mode

Its value comes from reducing false confidence around code edits by forcing more explicit awareness of:

* changed symbols
* changed relationships
* changed runtime assumptions
* applicable project rules
* targeted verification requirements

---

## 2. Core Implementation Principles

### 2.1 Non-Negotiable Principles

1. **Single-binary Rust CLI first**.
2. **Wrapper-first Gemini integration**.
3. **Repo-local state by default** under `.changeguard/`.
4. **Conservative, deterministic behavior** over speculative automation.
5. **Windows-first execution quality**.
6. **Cross-platform support via isolated adapters**, not scattered conditionals.
7. **Targeted verification**, not full-repo verification by default.
8. **Local-only state and analysis by default**.
9. **Graceful degradation** when partial features fail.
10. **Safe rebuildability** of all generated local state.

### 2.2 Explicit v1 Anti-Goals

Do not introduce the following in v1 unless directly required by a blocking implementation need:

* required Python runtime
* Tokio/async-first architecture
* HTTP service layer
* MCP server
* cloud sync/telemetry backend
* background daemon outside the active CLI process
* autonomous commit/rebase/cherry-pick flows
* unrestricted AI write execution
* deep whole-program semantic analysis beyond practical changed-file intelligence

---

## 3. Architecture Boundaries

The implementation must preserve separation between these subsystems:

1. **CLI routing**
2. **Platform detection and shell/path/env handling**
3. **Repo-local state management**
4. **Git scanning and diff analysis**
5. **Watcher and debounce batching**
6. **Language-aware indexing**
7. **Impact and risk scoring**
8. **Policy/rules evaluation**
9. **Verification planning and execution**
10. **Gemini prompt generation and wrapper invocation**

Implementers must not collapse these boundaries for convenience.

---

## 4. Hardened Repository Layout

```text
changeguard/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
├── docs/
│   ├── prd.md
│   ├── implementation-plan.md
│   ├── architecture.md
│   ├── upgrade-notes.md
│   └── examples/
│       ├── config.toml
│       ├── rules.toml
│       └── CHANGEGUARD.md
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── doctor.rs
│   │   ├── scan.rs
│   │   ├── watch.rs
│   │   ├── impact.rs
│   │   ├── verify.rs
│   │   ├── ask.rs
│   │   └── reset.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   ├── load.rs
│   │   ├── validate.rs
│   │   └── defaults.rs
│   ├── platform/
│   │   ├── mod.rs
│   │   ├── detect.rs
│   │   ├── shell.rs
│   │   ├── paths.rs
│   │   ├── env.rs
│   │   └── process_policy.rs
│   ├── state/
│   │   ├── mod.rs
│   │   ├── layout.rs
│   │   ├── db.rs
│   │   ├── migrations.rs
│   │   ├── reports.rs
│   │   └── locks.rs
│   ├── git/
│   │   ├── mod.rs
│   │   ├── repo.rs
│   │   ├── status.rs
│   │   ├── diff.rs
│   │   └── classify.rs
│   ├── watch/
│   │   ├── mod.rs
│   │   ├── debounce.rs
│   │   ├── filters.rs
│   │   ├── batch.rs
│   │   └── normalize.rs
│   ├── index/
│   │   ├── mod.rs
│   │   ├── symbols.rs
│   │   ├── references.rs
│   │   ├── runtime_usage.rs
│   │   ├── normalize.rs
│   │   ├── storage.rs
│   │   └── languages/
│   │       ├── mod.rs
│   │       ├── rust.rs
│   │       ├── typescript.rs
│   │       └── python.rs
│   ├── impact/
│   │   ├── mod.rs
│   │   ├── packet.rs
│   │   ├── score.rs
│   │   ├── relationships.rs
│   │   ├── reasoning.rs
│   │   └── redact.rs
│   ├── policy/
│   │   ├── mod.rs
│   │   ├── rules.rs
│   │   ├── matching.rs
│   │   ├── mode.rs
│   │   └── protected_paths.rs
│   ├── verify/
│   │   ├── mod.rs
│   │   ├── plan.rs
│   │   ├── runner.rs
│   │   ├── results.rs
│   │   └── timeouts.rs
│   ├── gemini/
│   │   ├── mod.rs
│   │   ├── modes.rs
│   │   ├── prompt.rs
│   │   ├── wrapper.rs
│   │   └── sanitize.rs
│   ├── output/
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   ├── table.rs
│   │   ├── diagnostics.rs
│   │   └── human.rs
│   └── util/
│       ├── mod.rs
│       ├── fs.rs
│       ├── hashing.rs
│       ├── process.rs
│       ├── text.rs
│       └── clock.rs
├── tests/
│   ├── cli_init.rs
│   ├── cli_doctor.rs
│   ├── cli_scan.rs
│   ├── cli_impact.rs
│   ├── cli_verify.rs
│   ├── state_db.rs
│   ├── gitignore_behavior.rs
│   ├── impact_packets.rs
│   ├── verification_plans.rs
│   ├── platform_windows.rs
│   ├── platform_wsl.rs
│   └── fixtures/
└── .github/
    └── workflows/
```

---

## 5. Repo-Local State Model

### 5.1 Default Location

All working state SHALL live under:

```text
.changeguard/
```

### 5.2 Working Layout

```text
.changeguard/
  config.toml
  rules.toml
  logs/
  tmp/
  reports/
    latest-scan.json
    latest-impact.json
    latest-verify.json
  state/
    current-batch.json
    tool-health.json
  db.sqlite3
  db.sqlite3-shm
  db.sqlite3-wal
```

### 5.3 Git Ignore Policy

`changeguard init` SHALL add `.changeguard/` to `.gitignore` by default unless disabled.

### 5.4 State Rebuildability

All derived state must be safe to delete and rebuild.

### 5.5 Reset Command

A `reset` command SHALL be available to:

* delete derived state
* preserve or optionally remove config/rules
* recover from DB corruption or bad cache state

---

## 6. Pinned Dependency Baseline

This section provides the approved starting dependency baseline for v1.

### 6.1 Toolchain

* **Rust**: 1.95.0+
* **Edition**: 2024
* **MSRV for v1**: 1.95.0

### 6.2 Approved Dependency Baseline

Use this as the initial dependency family and pinning approach.

```toml
[dependencies]
clap = { version = "4.6.1", features = ["derive"] }
clap_complete = "4.6.2"
clap_mangen = "0.2.31"

serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0"
toml = "1.1.2"

anyhow = "1.0.102"
miette = { version = "7.6.0", features = ["fancy"] }
thiserror = "2.0"

tracing = "0.1"
tracing-subscriber = { version = "0.3.20", features = ["fmt", "env-filter"] }

notify-debouncer-full = "0.7.0"
ignore = "0.4.25"
globset = "0.4.18"

camino = "1.2.2"
bstr = "1"

rusqlite = { version = "0.39.0", features = ["bundled"] }
rusqlite_migration = "2.5.0"

gix = "0.81.0"

blake3 = "1.8"
regex = "1.12"
once_cell = "1.21"
parking_lot = "0.12"

tree-sitter = "0.26.8"
tree-sitter-rust = "0.24.2"
tree-sitter-typescript = "0.23.2"
tree-sitter-python = "0.25.0"
```

### 6.3 Dependency Hardening Rules

* Commit `Cargo.lock`.
* Use `cargo audit` in CI.
* Use `cargo deny` in CI.
* Avoid adding dependencies without a concrete phase requirement.
* Avoid crates that require fragile native dependencies unless materially justified.
* Prefer `rusqlite` with `bundled` for Windows ease of installation.
* Treat tree-sitter crate family updates as coordinated changes, not casual bumps.

### 6.4 Explicit Dependency Cautions

1. **tree-sitter family**: exact compatibility must be proven with parser tests; do not assume “latest” implies smooth compatibility.
2. **watcher crates on Windows**: treat watcher behavior as empirically verified, not theoretically solved.
3. **shell/process helpers**: prefer direct process invocation APIs over command-string composition.
4. **gix**: use for repo inspection before resorting to shelling out to `git`; shell fallback should be explicit and rare.

---

## 7. Threat Model and Safety Posture

### 7.1 Safety Goals

* avoid accidental destructive commands
* avoid secret leakage in prompts/logs/reports
* avoid unbounded subprocesses
* avoid corrupted state becoming silently authoritative
* avoid user confusion around stale packets

### 7.2 Safety Defaults

* no unrestricted AI writes by default
* no auto-commit behavior
* redact likely secrets in reports/prompts
* bounded verification command execution
* timeout-aware subprocess handling
* local-only state unless the user explicitly exports data

### 7.3 Protected Paths

Support protected-path rules for locations such as:

* `.github/workflows/`
* `Cargo.toml`
* `package.json`
* `pyproject.toml`
* `.env*`
* `docker-compose*.yml`
* `infra/`
* `migrations/`

Changes to protected paths should increase risk and often require stronger verification or analyze-only mode.

---

## 8. Edge Cases to Design For

### 8.1 File Watching

* editor temp files and swap files
* rename storms
* atomic-save patterns
* rapid save bursts
* branch checkout churn
* mass file generation under `target/` or `node_modules/`
* deleting files during active batching
* path casing differences on Windows

### 8.2 Git State

* repo with no commits yet
* detached HEAD
* shallow clone
* nested git repos
* submodules
* worktrees
* dirty repo before Changeguard starts
* rename detection ambiguity

### 8.3 Platform

* PowerShell quoting edge cases
* paths with spaces
* non-UTF-8 process output
* CRLF/LF drift
* repo on NTFS, tool in WSL
* repo in WSL filesystem, tool invoked from Windows path context
* unavailable or mismatched `git` executable
* Gemini CLI installed in one environment but not the other

### 8.4 State and DB

* interrupted writes
* WAL/shm files left behind
* stale reports after crash
* partial migration application
* schema drift during development
* DB corruption or lock contention

### 8.5 Language Analysis

* parse failure for syntactically incomplete files during editing
* unsupported file types inside a changed batch
* comments and string literals that look like env/config accesses
* generated code
* very large files
* mixed-language repos with partial support only

### 8.6 Verification

* commands that hang waiting for input
* commands that mutate state unexpectedly
* commands missing from PATH
* test frameworks returning huge output
* flaky tests
* repo-specific scripts with shell assumptions

### 8.7 Gemini Integration

* Gemini CLI absent
* Gemini CLI exits non-zero
* malformed prompt assembly
* packet too large
* stale packet vs current repo state mismatch
* accidental prompt inclusion of secrets or large irrelevant logs

---

## 9. High-Level Delivery Sequence

The implementation should proceed in the following order:

1. bootstrap the CLI
2. establish repo-local state model
3. implement init + gitignore behavior
4. implement doctor + platform detection
5. implement config/rules loading
6. implement one-shot scan with git metadata
7. implement basic impact packet shell
8. implement watch mode with reliable batching
9. implement language-aware indexing
10. implement relationship/runtime extraction
11. implement deterministic risk scoring
12. implement verification planning
13. implement verification runner
14. implement Gemini wrapper
15. formalize DB-backed persistence and recovery
16. harden Windows/WSL/Linux behavior
17. document, package, and stabilize

Important: a **basic impact packet shell** should exist before deep indexing is finished so the workflow stays end-to-end testable early.

---

## Phase 1: Bootstrap the CLI Skeleton

### Objective

Create a buildable, testable Rust CLI with stable command routing and diagnostics.

### Deliverables

* `main.rs`
* `cli.rs`
* subcommand scaffolding
* logging setup
* diagnostics plumbing

### Acceptance Criteria

* `changeguard --help` works
* subcommands render clearly
* `cargo build` passes on Windows and Linux
* `cargo test` passes with placeholder coverage

### Hardening Notes

* fail with human-readable diagnostics, not debug panics
* ensure UTF-8 assumptions are not hardcoded in all output handling

### Verification Gate

* `cargo fmt --check`
* `cargo clippy --all-targets --all-features`
* `cargo test`

---

## Phase 2: Repo-Local State Layout and Init

### Objective

Create `.changeguard/`, starter config/rules, and `.gitignore` integration.

### Deliverables

* state layout abstraction
* `init` command
* `.gitignore` updater
* starter config/rules files
* optional starter `CHANGEGUARD.md`

### Functional Requirements

* detect git repo presence
* create state dir idempotently
* append `.changeguard/` to `.gitignore` only if missing
* preserve existing `.gitignore` content as much as possible
* support `--no-gitignore`

### Edge Cases

* no `.gitignore` file exists
* `.gitignore` already contains `.changeguard/`
* read-only `.gitignore`
* repo root discovery from nested working directory

### Acceptance Criteria

* rerunning `init` is safe
* `.gitignore` updates are minimal and deterministic
* errors around repo discovery or write failure are clear

### Verification Gate

* unit tests for ignore file mutation
* fixture tests for multiple `.gitignore` shapes

---

## Phase 3: Doctor and Platform Detection

### Objective

Identify runtime environment quality before deeper functionality is used.

### Deliverables

* `doctor` command
* platform detection
* shell detection
* executable discovery
* WSL detection
* path classification

### Functional Requirements

* detect Windows/Linux/WSL
* detect preferred shell
* verify `git`
* verify Gemini CLI
* classify repo location semantics where feasible
* warn about likely Windows/WSL mismatch issues

### Edge Cases

* Gemini installed only in WSL
* repo on `/mnt/c/` from WSL
* repo on NTFS with PowerShell invoking Linux-targeted scripts
* shell cannot be reliably inferred

### Acceptance Criteria

* doctor output is useful, not noisy
* absent tools are reported clearly
* likely mixed-environment hazards are surfaced

### Verification Gate

* platform detection tests
* fake executable path tests

---

## Phase 4: Config and Rule Loading

### Objective

Implement deterministic config and policy loading.

### Deliverables

* config defaults
* loader
* validator
* rule matcher
* mode model

### Functional Requirements

* load built-in defaults first
* load repo-local config if present
* validate malformed files cleanly
* support path-based rule overrides
* support protected paths and required verifications

### Edge Cases

* malformed TOML
* unknown rule fields
* conflicting path rules
* missing config files

### Acceptance Criteria

* config behavior is deterministic
* invalid config never causes silent fallback without warning

### Verification Gate

* config parse tests
* rule precedence tests

---

## Phase 5: Git Scan Foundation

### Objective

Implement one-shot repository scan and basic change classification.

### Deliverables

* repo discovery
* git status collection
* diff summary collection
* branch/HEAD metadata
* clean vs dirty handling

### Functional Requirements

* identify changed files
* classify create/modify/delete/rename where feasible
* record staged vs unstaged state
* produce a basic scan report

### Edge Cases

* unborn branch / no commits
* detached HEAD
* shallow clone
* worktree
* submodule boundaries
* ignored-but-changed generated artifacts

### Acceptance Criteria

* clean repos are handled gracefully
* dirty repos produce useful summaries
* partial git limitations are surfaced honestly

### Verification Gate

* fixture-based repo tests
* clean/dirty repo tests

---

## Phase 6: Basic Impact Packet Shell

### Objective

Create the initial packet/report structure before deep indexing.

### Deliverables

* packet schema
* basic packet generation from git scan
* report writer
* `impact` command baseline

### Functional Requirements

* include repo metadata
* include changed files
* include provisional risk shell
* write JSON report locally

### Why Early

This keeps the end-to-end loop visible even before deeper indexing is complete.

### Acceptance Criteria

* `scan` + `impact` can produce a useful packet even without AST enrichment

### Verification Gate

* golden JSON snapshot tests

---

## Phase 7: Watch Mode and Batch Debouncing

### Objective

Build stable watch mode with change batching.

### Deliverables

* watcher initialization
* debounce strategy
* event filters
* batch persistence

### Functional Requirements

* watch supported source files recursively
* ignore build and temp churn
* merge related save events into one batch
* tolerate rapid edits and renames

### Edge Cases

* repeated saves on same file
* editors that save via temp + rename
* branch switching while watcher active
* deleted file after event capture but before analysis

### Acceptance Criteria

* watch mode remains stable during normal editing
* false escalations from temp-file churn are low

### Verification Gate

* tempdir integration tests
* Windows-specific rename/save tests

---

## Phase 8: Language-Aware Symbol Extraction

### Objective

Extract useful symbol metadata from changed Rust, TypeScript, and Python files.

### Deliverables

* language dispatch
* Rust parser
* TypeScript parser
* Python parser
* symbol storage model

### Functional Requirements

* parse changed files where possible
* extract top-level declarations and notable symbol data
* mark public/exported symbols where derivable
* detect probable signature changes when possible

### Edge Cases

* incomplete file during active edit
* syntax errors
* very large files
* generated/minified files

### Acceptance Criteria

* parser failure does not crash the scan
* changed symbols appear in packet enrichment when available

### Verification Gate

* fixture-based parsing tests for each language

---

## Phase 9: Relationships and Runtime Usage

### Objective

Add lightweight impact enrichment around imports, exports, env vars, config keys, and related usage.

### Deliverables

* import/export summaries
* runtime usage scanners
* env/config extraction
* relationship annotations in packet

### Functional Requirements

* detect import/export drift where feasible
* detect env var usage
* detect likely config key usage
* record related runtime assumptions touched

### Edge Cases

* false positives in comments or string literals
* dynamic access patterns
* partial file parse success

### Acceptance Criteria

* runtime-impact clues appear when applicable
* false positives are limited and explainable

### Verification Gate

* fixture tests for env/config detection across languages

---

## Phase 10: Deterministic Risk Scoring

### Objective

Assign LOW/MEDIUM/HIGH risk with explainable reasons.

### Deliverables

* score model
* reasoning strings
* protected-path influence
* packet risk enrichment

### Functional Requirements

* elevate public API changes
* elevate env/config changes
* elevate protected path changes
* elevate cross-language contract changes when detected
* prefer explainability over opaque scoring

### Edge Cases

* conflicting signals
* sparse data due to partial parse failure
* large batch across unrelated files

### Acceptance Criteria

* risk reasons are understandable
* same inputs yield same outputs

### Verification Gate

* scoring unit tests
* golden packet tests

---

## Phase 11: Verification Planning

### Objective

Create targeted verification plans from rules, language signals, and risk tier.

### Deliverables

* verification plan model
* rule-driven command selection
* path/language/risk plan logic

### Functional Requirements

* prefer targeted checks
* escalate on HIGH risk
* support empty/no-op plans gracefully
* support project-specific required commands

### Edge Cases

* no configured commands for repo language
* conflicting rules
* repo scripts available only in one environment

### Acceptance Criteria

* plan generation is deterministic and inspectable

### Verification Gate

* fixture tests for plan generation

---

## Phase 12: Verification Runner

### Objective

Execute checks safely and capture structured results.

### Deliverables

* subprocess runner
* timeout support
* output capture
* result persistence
* report generation

### Functional Requirements

* execute commands without brittle shell composition when possible
* capture exit code, duration, stdout/stderr snippets
* enforce timeouts
* prevent hanging interactive prompts where feasible

### Edge Cases

* missing executable
* hanging command
* command that mutates the repo
* huge output volume
* non-UTF-8 output

### Acceptance Criteria

* failures are surfaced cleanly
* result records remain usable even on partial failure

### Verification Gate

* tests with fake success/failure/hang commands

---

## Phase 13: Gemini Wrapper Integration

### Objective

Invoke Gemini CLI using the current structured packet and selected prompt mode.

### Deliverables

* prompt rendering
* mode handling
* wrapper invocation
* sanitized context inclusion

### Functional Requirements

* support `analyze`, `suggest`, `review_patch`
* include impact packet
* optionally include verification results
* redact likely secrets
* fail gracefully when Gemini is absent

### Edge Cases

* stale packet vs current repo state
* prompt too large
* Gemini executable exits non-zero
* packet includes noisy irrelevant output

### Acceptance Criteria

* Gemini invocation is inspectable and bounded
* prompt generation is debuggable

### Verification Gate

* prompt rendering tests
* subprocess fake executable tests

---

## Phase 14: DB-Backed Persistence, Migrations, and Recovery

### Objective

Formalize SQLite-backed persistence for analysis history and current state.

### Deliverables

* schema
* migrations
* storage API
* DB recovery behavior
* reset behavior

### Suggested Schema Domains

* `batches`
* `changed_files`
* `symbols`
* `impact_packets`
* `verification_runs`
* `verification_results`

### Functional Requirements

* auto-initialize DB when needed
* use migrations for schema evolution
* detect DB corruption and provide reset/rebuild path
* keep JSON exports even when DB is used

### Edge Cases

* interrupted migration
* stale WAL files
* locked DB
* schema mismatch from earlier dev build

### Acceptance Criteria

* DB is not a single point of silent failure
* state can be rebuilt safely

### Verification Gate

* migration tests
* corruption-recovery tests
* reset tests

---

## Phase 15: Cross-Platform Hardening

### Objective

Close real-world gaps across Windows, WSL, and Ubuntu.

### Deliverables

* path normalization hardening
* process launching hardening
* shell policy hardening
* line-ending tolerance improvements
* mixed-environment diagnostics

### Functional Requirements

* normalize internal repo-relative paths consistently
* avoid fragile shell command composition
* detect likely environment mismatch problems
* survive spaces in paths and non-UTF-8 output

### Manual Test Matrix

* Windows PowerShell, repo on NTFS
* WSL Ubuntu, repo in Linux filesystem
* WSL Ubuntu, repo on mounted Windows drive
* Gemini installed in host only
* Gemini installed in WSL only

### Acceptance Criteria

* core commands behave predictably across tested environments

### Verification Gate

* path conversion fixtures
* platform-specific manual validation checklist

---

## Phase 16: Documentation, Packaging, and Release Readiness

### Objective

Make the tool ready for repeated use across multiple repositories.

### Deliverables

* README
* quickstart
* example config/rules files
* architecture notes
* upgrade notes
* release checklist
* CI workflows

### Functional Requirements

* document repo-local state and rebuild behavior
* document `.gitignore` default behavior
* document Windows/WSL caveats
* document safe usage expectations

### Acceptance Criteria

* a new user can install, init, doctor, scan, impact, and verify without guesswork

### Verification Gate

* smoke-run the README flow on a clean sample repo

---

## 10. Milestones

### Milestone A — Core Local CLI

Complete:

* Phase 1
* Phase 2
* Phase 3
* Phase 4
* Phase 5
* Phase 6

### Milestone B — Intelligent Local Analysis

Complete:

* Phase 7
* Phase 8
* Phase 9
* Phase 10

### Milestone C — Safe Verification and Gemini Assist

Complete:

* Phase 11
* Phase 12
* Phase 13

### Milestone D — Durable Tooling

Complete:

* Phase 14
* Phase 15
* Phase 16

---

## 11. Testing Strategy

### Unit Tests

Use for:

* config parsing
* rule matching
* path normalization
* risk scoring
* `.gitignore` mutation
* prompt rendering

### Fixture Tests

Use for:

* git state classification
* language parsing
* env/config detection
* verification planning

### Integration Tests

Use for:

* `init`
* `doctor`
* `scan`
* `impact`
* `verify`
* `reset`

### Manual Validation

Always validate on:

* Windows 11 + PowerShell
* WSL2 Ubuntu
* Ubuntu native if available

### CI Recommendations

Run at minimum:

* `cargo fmt --check`
* `cargo clippy --all-targets --all-features`
* `cargo test`
* `cargo audit`
* `cargo deny check`

---

## 12. AI Implementation Protocol

Each AI implementation pass should follow this discipline:

1. Implement one phase or tightly bounded subphase only.
2. Add or update tests in the same pass.
3. Run format, lint, and test gates.
4. Document deviations explicitly.
5. Do not silently introduce architecture from future phases.
6. Prefer partial working behavior over speculative cleverness.

---

## 13. Recommended Default v1 Behavior

Changeguard v1 should default to:

* repo-local state in `.changeguard/`
* automatic `.gitignore` update for `.changeguard/`
* one-shot scan before advanced watch-based workflows
* conservative risk scoring
* targeted verification
* wrapper-only Gemini integration
* no unrestricted AI writes by default
* explicit recovery path when state becomes stale or broken

---

## 14. Final Implementation Warning

The most likely way to fail this project is to make it “smart” before it is stable.

The most important success criteria are:

* predictable behavior
* inspectable outputs
* safe local state
* clear platform handling
* bounded subprocess execution
* explainable impact and risk analysis

Reliability comes first. Sophistication comes second.

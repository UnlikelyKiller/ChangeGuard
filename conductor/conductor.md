# ChangeGuard Conductor

## Milestone L: Ledger Incorporation (Completed)

*   **Track L1-1: Ledger Data Model & Migrations**
    *   Status: Completed
    *   Spec: `conductor/trackL1-1/spec.md`
    *   Plan: `conductor/trackL1-1/plan.md`
    *   Goal: Implement Phase L1 (Transaction Lifecycle & Data Model) - Data Model, Error types, and Migrations M11 & M12.
    *   Key additions: `src/ledger/types.rs`, `src/ledger/error.rs`, SQLite migrations M11-M12, `LedgerConfig`.

*   **Track L1-2: Transaction Lifecycle Management**
    *   Status: Completed
    *   Spec: `conductor/trackL1-2/spec.md`
    *   Plan: `conductor/trackL1-2/plan.md`
    *   Goal: Implement the core transaction lifecycle (start, commit, rollback, atomic) and CLI commands.
    *   Key additions: `src/ledger/db.rs`, `src/ledger/transaction.rs`, `src/ledger/session.rs`, `ledger start/commit/rollback/atomic`.

*   **Track L1-R: Ledger Phase 1 Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackL1-R/spec.md`
    *   Plan: `conductor/trackL1-R/plan.md`
    *   Goal: Address the high and medium severity findings from the Codex review for Phase L1.
    *   Key additions: Transactional safety, correct lifecycle states, robust path normalization, WAL concurrency, verification persistence, and CLI gaps.

*   **Track L2-1: Ledger Drift Detection**
    *   Status: Completed
    *   Spec: `conductor/trackL2-1/spec.md`
    *   Plan: `conductor/trackL2-1/plan.md`
    *   Goal: Integrate file watcher with the ledger to detect and record untracked changes (drift).
    *   Key additions: Watcher transaction checks, UNAUDITED record creation, drift counting, enhanced status reporting.

*   **Track L2-2: Ledger Reconciliation & Adoption**
    *   Status: Completed
    *   Spec: `conductor/trackL2-2/spec.md`
    *   Plan: `conductor/trackL2-2/plan.md`
    *   Goal: Implement reconciliation and adoption commands to manage detected drift.
    *   Key additions: `ledger reconcile`, `ledger adopt`, drift transition logic, reconciliation provenance.

*   **Track L3-1: Enforcement Data Model & Registration**
    *   Status: Completed
    *   Spec: `conductor/trackL3-1/spec.md`
    *   Plan: `conductor/trackL3-1/plan.md`
    *   Goal: Implement data model and CLI for tech stack enforcement and commit validators.
    *   Key additions: Enforcement enums/types, Migration M13, `ledger register` and `ledger stack` commands.

*   **Track L3-R: Enforcement Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackL3-R/spec.md`
    *   Plan: `conductor/trackL3-R/plan.md`
    *   Goal: Address Codex findings for Track L3-1: JSON defaults, CLI alignment, FK enforcement, and filtering.
    *   Key additions: Serde defaults, flagged CLI args, category filtering, SQLite FK pragma.

*   **Track L3-2: Enforcement & Validation Logic**
    *   Status: Completed
    *   Spec: `conductor/trackL3-2/spec.md`
    *   Plan: `conductor/trackL3-2/plan.md`
    *   Goal: Implement active tech stack enforcement and commit-time validator execution.
    *   Key additions: `NO <term>` check at start, validator runner (shell execution, timeouts, {entity} substitution), lifecycle integration.

*   **Track L3-R2: Enforcement Logic Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackL3-R2/spec.md`
    *   Plan: `conductor/trackL3-R2/plan.md`
    *   Goal: Address Codex findings for Track L3-2: absolute path substitution, global validator inclusion, and specific error variants.
    *   Key additions: Absolute entity path in validators, 'ALL' category support in DB queries, RuleViolation and ValidatorFailed errors.

*   **Track L4-1: Transaction-Linked ADR Generation**
    *   Status: Completed
    *   Spec: `conductor/trackL4-1/spec.md`
    *   Plan: `conductor/trackL4-1/plan.md`
    *   Goal: Implement the `ledger adr` command to export architectural decisions as MADR-format markdown.
    *   Key additions: `ledger adr` command, MADR template, entry fetching for architecture/breaking changes.

*   **Track L4-2: FTS5 Search Integration**
    *   Status: Completed
    *   Spec: `conductor/trackL4-2/spec.md`
    *   Plan: `conductor/trackL4-2/plan.md`
    *   Goal: Implement the `ledger search` command using SQLite FTS5 for full-text search across ledger entries.
    *   Key additions: `ledger search` command, FTS5 query logic in DB, ranked search results.

*   **Track L4-R: Search & ADR Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackL4-R/spec.md`
    *   Plan: `conductor/trackL4-R/plan.md`
    *   Goal: Address Codex findings for Phase L4: FTS alias fix, timestamp format alignment, and MADR template completion.
    *   Key additions: `f MATCH` query, RFC3339 date comparison, ADR `## Decision` section, `TransactionManager` search wrapper.

*   **Track L5-1: Token-Level Provenance**
    *   Status: Completed
    *   Spec: `conductor/trackL5-1/spec.md`
    *   Plan: `conductor/trackL5-1/plan.md`
    *   Goal: Implement token-level attribution to transactions, recording symbol modifications in the ledger.
    *   Key additions: `token_provenance` table (M14), symbol attribution logic, symbol-level history in audit.

*   **Track L6-1: Ledger Federation**
    *   Status: Completed
    *   Spec: `conductor/trackL6-1/spec.md`
    *   Plan: `conductor/trackL6-1/plan.md`
    *   Goal: Implement cross-repo ledger federation, exporting local entries and importing sibling entries.
    *   Key additions: `ledger` array in `schema.json`, federated audit/impact views, [FEDERATED] markings.

*   **Track L6-R: Ledger Federation Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackL6-R/spec.md`
    *   Plan: `conductor/trackL6-R/plan.md`
    *   Goal: Address Codex findings for Track L6-1: correct federation identity, path confinement, export limit, and impact query.
    *   Key additions: `origin = SIBLING`, `trace_id = sibling_name`, 30-day export limit, local DB impact query for siblings.

*   **Track L7-1: Production Polish**
    *   Status: Completed
    *   Spec: `conductor/trackL7-1/spec.md`
    *   Plan: `conductor/trackL7-1/plan.md`
    *   Goal: Polish the Ledger implementation for production readiness: UI enhancements, actionable errors, and complete documentation.
    *   Key additions: Color-coded icons, refined miette errors, comprehensive README and skill documentation.

*   **Track L-H1: Ledger Production Hardening**
    *   Status: Completed
    *   Spec: `conductor/trackL-H1/spec.md`
    *   Plan: `conductor/trackL-H1/plan.md`
    *   Goal: Address critical and high-severity Codex findings: lifecycle invariants, durable state protection, and secure path normalization.
     *   Key additions: Unique PENDING index, conditional status updates, --include-ledger reset flag, lexical path normalization utility, ProcessPolicy for validators.


## Milestone M: Observability & Intelligence Expansion (Completed)

*   **Track M1-1: Embedding HTTP Client & SQLite Schema**
    *   Status: Completed
    *   Spec: `conductor/trackM1-1/spec.md`
    *   Plan: `conductor/trackM1-1/plan.md`
    *   Goal: Establish embedding infrastructure: config model, SQLite migrations for 5 new tables, HTTP client for local embedding model, content-addressed vector storage.
    *   Key additions: `LocalModelConfig`/`DocsConfig`/`ObservabilityConfig`/`ContractsConfig`, `embeddings`/`doc_chunks`/`api_endpoints`/`test_outcome_history`/`observability_snapshots` tables, `src/embed/client.rs` & `storage.rs`.

*   **Track M1-2: Cosine Similarity, Top-K, Budget & Doctor**
    *   Status: Completed
    *   Spec: `conductor/trackM1-2/spec.md`
    *   Plan: `conductor/trackM1-2/plan.md`
    *   Goal: Complete embedding primitives with cosine similarity, top-k retrieval, token budget enforcement, embed_and_store convenience, chunking+mean-pool for long texts, and doctor health reporting.
    *   Key additions: `src/embed/similarity.rs`, `src/embed/budget.rs`, `embed_long_text()`, `embed_and_store()`, doctor local model status check.

*   **Track M2-1: Document Crawler & Chunker**
    *   Status: Completed
    *   Spec: `conductor/trackM2-1/spec.md`
    *   Plan: `conductor/trackM2-1/plan.md`
    *   Goal: Implement document indexing pipeline: walk configured docs paths, split into semantic chunks, store in `doc_chunks` table.
    *   Key additions: `src/docs/crawler.rs`, `src/docs/chunker.rs`, `src/docs/index.rs`, `changeguard index --docs` flag.

*   **Track M2-2: Retrieval, Reranking & Impact Enrichment**
    *   Status: Completed
    *   Dependencies: M1-2, M2-1
    *   Spec: `conductor/trackM2-2/spec.md`
    *   Plan: `conductor/trackM2-2/plan.md`
    *   Goal: Wire indexed doc chunks into impact analysis: semantic retrieval, reranking, `relevant_decisions` in ImpactPacket, and ask context injection.
    *   Key additions: `src/retrieval/query.rs`, `src/retrieval/rerank.rs`, `RelevantDecision` type, impact enrichment, ask context extension.

*   **Track M3-1: Local Model Client & Context Assembly**
    *   Status: Completed
    *   Dependencies: M1-2
    *   Spec: `conductor/trackM3-1/spec.md`
    *   Plan: `conductor/trackM3-1/plan.md`
    *   Goal: Build OpenAI-compatible completions client for llama-server and context assembly pipeline for prompts.
    *   Key additions: `src/local_model/client.rs`, `src/local_model/context.rs`, completions endpoint, token-budgeted context assembly.

*   **Track M3-2: Ask Backend Routing & Integration**
    *   Status: Completed
    *   Dependencies: M3-1
    *   Spec: `conductor/trackM3-2/spec.md`
    *   Plan: `conductor/trackM3-2/plan.md`
    *   Goal: Wire local model into `changeguard ask` with `--backend` flag, auto-selection logic, and `config verify` extension.
    *   Key additions: `Backend` enum, `--backend local/gemini` flag, `resolve_backend()` auto-selection, `config verify` backend reporting.

*   **Track M4-1: Test Outcome Recording & Diff Embedding**
    *   Status: Completed
    *   Dependencies: M1-2
    *   Spec: `conductor/trackM4-1/spec.md`
    *   Plan: `conductor/trackM4-1/plan.md`
    *   Goal: Build data collection for semantic test prediction: embed diffs after verify runs, store test outcomes linked to embeddings.
    *   Key additions: `src/verify/semantic_predictor.rs`, `TestOutcome` enum, `record_test_outcomes()`, hook into `execute_verify()`.

*   **Track M4-2: Semantic Predictor & Score Blending**
    *   Status: Completed
    *   Dependencies: M4-1
    *   Spec: `conductor/trackM4-2/spec.md`
    *   Plan: `conductor/trackM4-2/plan.md`
    *   Goal: Implement semantic prediction: query past test outcomes by diff similarity, blend with rule-based scores, surface via `--explain`.
    *   Key additions: `compute_semantic_scores()`, `semantic_weight` config, score blending in predictor, `--explain` flag.

*   **Track M5-1: Prometheus Client & Log Scanner**
    *   Status: Completed
    *   Dependencies: M1-2
    *   Spec: `conductor/trackM5-1/spec.md`
    *   Plan: `conductor/trackM5-1/plan.md`
    *   Goal: Build observability fetching infrastructure: PromQL query client, local log file scanner, ObservabilitySignal type, snapshot storage.
    *   Key additions: `src/observability/prometheus.rs`, `src/observability/log_scanner.rs`, `src/observability/signal.rs`, Prometheus query client.

*   **Track M5-2: Observability Impact Enrichment**
    *   Status: Completed
    *   Dependencies: M5-1
    *   Spec: `conductor/trackM5-2/spec.md`
    *   Plan: `conductor/trackM5-2/plan.md`
    *   Goal: Wire observability signals into impact analysis: fetch live signals, elevate risk on threshold breach, populate ImpactPacket, inject into ask context.
    *   Key additions: `enrich_observability()`, risk elevation from observability signals, `observability` field in ImpactPacket, ask context injection.

*   **Track M6-1: OpenAPI Spec Parser & Index Storage**
    *   Status: Completed
    *   Dependencies: M1-2
    *   Spec: `conductor/trackM6-1/spec.md`
    *   Plan: `conductor/trackM6-1/plan.md`
    *   Goal: Build OpenAPI/Swagger spec parsing and indexing: parse specs into endpoints, embed descriptions, store in `api_endpoints`.
    *   Key additions: `src/contracts/parser.rs`, `src/contracts/index.rs`, `serde_yaml` dependency, `changeguard index --contracts` flag.

*   **Track M6-2: Contract Matching & Impact Enrichment**
    *   Status: Completed
    *   Dependencies: M6-1
    *   Spec: `conductor/trackM6-2/spec.md`
    *   Plan: `conductor/trackM6-2/plan.md`
    *   Goal: Match changed files to API endpoints via embedding similarity, flag public contract risk, surface in ImpactPacket and human output.
     *   Key additions: `src/contracts/matcher.rs`, `AffectedContract` type, contract matching in impact, human output table, ask context extension.


## Milestone M7: Engineering Coverage Deepening (Planning)

*   **Track M7-1: Trace Config & SDK Dependency Detection**
    *   Status: Completed
    *   Spec: `conductor/trackM7-1/spec.md`
    *   Plan: `conductor/trackM7-1/plan.md`
    *   Goal: Detect observability pipeline config changes (otel-collector, Jaeger, DataDog, Grafana Agent) and third-party SDK import additions (Stripe, Auth0, Twilio, etc.).
    *   Key Additions: `src/coverage/traces.rs`, `src/coverage/sdk.rs`, Go support in `src/index/references.rs`, `CoverageConfig` in `src/config/model.rs`.

    *   Key additions: `src/coverage/traces.rs`, `src/coverage/sdk.rs`, `TraceConfigChange` type, `SdkDependencyDelta` type.

*   **Track M7-2: Service-Map Derivation**
    *   Status: Completed
    *   Spec: `docs/observability-plan2.md` §5
    *   Plan: `conductor/trackM7-2/plan.md`
    *   Goal: Infer service boundaries from route/handler/data-model topology, derive cross-service dependency edges.
    *   Key Additions: `src/coverage/services.rs`, migration M15, `service_map_delta` in `ImpactPacket`, Go support in extension lists.

*   **Track M7-3: Data-Flow Coupling Risk**
    *   Status: Planning
    *   Dependencies: M7-2
    *   Spec: `docs/observability-plan2.md` §6
    *   Plan: `conductor/trackM7-3/plan.md`
    *   Goal: Flag call chains where route handlers and their data models co-change, detect incomplete refactors.
    *   Key additions: `src/coverage/dataflow.rs`, `DataFlowMatch` type, cycle detection, change-percentage threshold.

*   **Track M7-4: Deployment Manifest Awareness**
    *   Status: Planning
    *   Spec: `docs/observability-plan2.md` §7
    *   Plan: `conductor/trackM7-4/plan.md`
    *   Goal: Classify Dockerfile, docker-compose, k8s, terraform, and helm changes with tiered risk weighting.
    *   Key additions: `src/coverage/deploy.rs`, `ManifestType` enum, `DeployManifestChange` type, Dockerfile COPY/ADD scanning.

*   **Track M7-5: CI Pipeline Self-Awareness**
    *   Status: Planning
    *   Spec: `docs/observability-plan2.md` §8
    *   Plan: `conductor/trackM7-5/plan.md`
    *   Goal: Surface risk when CI config itself changes in a diff, detect CI+source co-change patterns.
    *   Key additions: Extend `src/index/ci_gates.rs`, pre-commit hook awareness.

*   **Track M7-6: ADR Staleness Detection**
    *   Status: Planning
    *   Dependencies: M2-2
    *   Spec: `docs/observability-plan2.md` §9
    *   Plan: `conductor/trackM7-6/plan.md`
    *   Goal: Flag retrieved ADRs exceeding configurable age threshold with severity tiers and recently-updated exemption.
    *   Key additions: Extend `RelevantDecision` with `staleness_days`, multi-source age detection.

*   **Track M7-7: Impact Packet Extension & Enrichment Integration**
    *   Status: Planning
    *   Dependencies: M7-1..M7-6
    *   Spec: `docs/observability-plan2.md` §10
    *   Plan: `conductor/trackM7-7/plan.md`
    *   Goal: Wire all M7 detection into ImpactPacket, risk scoring, human output, and ask context. Master kill switch via `[coverage].enabled`.
    *   Key additions: 5 new ImpactPacket fields, 7 enrichment hooks, 7 human output sections, per-dimension kill switches.


## Milestone J: Phase 2 Final Remediation (Completed)

*   **Track 36: Critical Remediation & Green CI**
    *   Status: Completed
    *   Spec: `conductor/track36/spec.md`
    *   Plan: `conductor/track36/plan.md`
    *   Goal: Restore `cargo fmt` and `cargo clippy`, fix silent hotspot row error dropping, and surface prediction degradation in `verify`.
    *   Key additions: Green CI pipeline, visible SQLite hotspot errors, user-visible verification prediction warnings.

*   **Track 37: LSP Daemon Functional Completion**
    *   Status: Completed
    *   Spec: `conductor/track37/spec.md`
    *   Plan: `conductor/track37/plan.md`
    *   Goal: Implement real Hover, CodeLens, real-time diagnostics, broken-stdin self-termination, and lifecycle tests, removing all placeholders.
    *   Key additions: Fully functional LSP handlers, `data_stale` surface handling, stdin auto-termination, robust daemon lifecycle tests.

*   **Track 38: Complexity & Temporal Hardening**
    *   Status: Completed
    *   Spec: `conductor/track38/spec.md`
    *   Plan: `conductor/track38/plan.md`
    *   Goal: Complete missing edge tests for complexity, add real temporal git fixtures, add dedicated hotspot tests, and document the arborist spike.
    *   Key additions: Arborist ADR, TS/syntax/unsupported-language complexity tests, temporal git-fixture testing, hotspot scoring tests.

*   **Track 39: Dependency & Federation Deepening**
    *   Status: Completed
    *   Spec: `conductor/track39/spec.md`
    *   Plan: `conductor/track39/plan.md`
    *   Goal: Upgrade structural prediction and dependency discovery to use current repo data, automate federation discovery, and use shared redaction.
    *   Key additions: Current-repo structural analysis, automated federation discovery in `scan`/`impact`, shared redaction model usage, path confinement edge tests.

*   **Track 40: Narrative Refinement**
    *   Status: Completed
    *   Spec: `conductor/track40/spec.md`
    *   Plan: `conductor/track40/plan.md`
    *   Goal: Rework narrative prompt construction to avoid nesting and ensure fallback write failures are visible.
    *   Key additions: Flat, structured narrative prompts, robust error handling for fallback report persistence.

## Milestone I: Phase 2 Remediation (Completed)

*   **Track 30: Foundation & Safety Remediation**
    *   Status: Completed
    *   Spec: `conductor/track30/spec.md`
    *   Plan: `conductor/track30/plan.md`
    *   Goal: Restore green CI gates (fmt, clippy, tests), fix critical secret redaction/persistence issues, and remove production unwraps.
    *   Key additions: Green CI suite, secured redact-before-save persistence, zero production unwraps in impact/hotspots.

*   **Track 31: Intelligence & Determinism Hardening**
    *   Status: Completed
    *   Spec: `conductor/track31/spec.md`
    *   Plan: `conductor/track31/plan.md`
    *   Goal: Fix temporal first-parent traversal, add complexity degradation, and ensure deterministic sorting for hotspots.
    *   Key additions: First-parent traversal by default, complexity AST degradation, multiplication-based hotspot scoring, hotspot JSON/filtering.

*   **Track 32: Predictive Verification Completion**
    *   Status: Completed
    *   Spec: `conductor/track32/spec.md`
    *   Plan: `conductor/track32/plan.md`
    *   Goal: Implement structural prediction (removing placeholders) and add missing predictor tests and degradation warnings.
    *   Key additions: Structural import-based prediction, placeholder cleanup, comprehensive predictor tests, merged plan descriptions.

*   **Track 33: Federated Intelligence Completion**
    *   Status: Completed
    *   Spec: `conductor/track33/spec.md`
    *   Plan: `conductor/track33/plan.md`
    *   Goal: Implement real cross-repo impact resolution with dependency edges, schema validation, and path confinement.
    *   Key additions: Path confinement security, schema validation, dependency insertion, cross-repo impact resolution tests.

*   **Track 34: Narrative Reporting Completion**
    *   Status: Completed
    *   Spec: `conductor/track34/spec.md`
    *   Plan: `conductor/track34/plan.md`
    *   Goal: Wire token budgeting and truncation annotations into Gemini execution, and add golden prompt tests.
    *   Key additions: Strict token budgeting (409,600 chars), truncation annotations, robust Gemini execution with fallback artifacts, byte-for-byte deterministic prompt tests.

*   **Track 35: LSP Daemon Resolution**
    *   Status: Completed
    *   Spec: `conductor/track35/spec.md`
    *   Plan: `conductor/track35/plan.md`
    *   Goal: Replace the stub implementation with a fully-featured LSP server using `tower-lsp-server` and `tokio`.
    *   Key additions: Real-time diagnostic reporting, robust lifecycle (PID management), read-only WAL SQLite access, and native async trait implementation.

## Completed Tracks


*   **Track 19: Reset and Recovery Completion**
    *   Status: Completed
    *   Spec: `conductor/track19/spec.md`
    *   Plan: `conductor/track19/plan.md`
    *   Audit2 findings: Functional finding 1, Phase 14 gap, Source-tree `reset` deficiency
    *   Key additions: real `reset` command, `src/commands/reset.rs`, derived-state cleanup, optional config/rules removal, recovery path for broken local state

*   **Track 20: Determinism and Error Visibility Hardening**
    *   Status: Completed
    *   Spec: `conductor/track20/spec.md`
    *   Plan: `conductor/track20/plan.md`
    *   Audit2 findings: Functional findings 3 and 4, Determinism gaps, Error Visibility gaps
    *   Key additions: validated rule loading, no silent config/rules fallback, explicit partial-analysis warnings in impact packets, deterministic warning ordering

*   **Track 21: Verification Process Hardening**
    *   Status: Completed
    *   Spec: `conductor/track21/spec.md`
    *   Plan: `conductor/track21/plan.md`
    *   Audit2 findings: Functional finding 2, Phase 12 and Phase 15 gaps
    *   Key additions: `verify/runner.rs`, `verify/timeouts.rs`, process-policy enforcement, reduced shell dependence, dedicated platform verification tests

*   **Track 22: Structural Completion and Plan Reconciliation**
    *   Status: Completed
    *   Spec: `conductor/track22/spec.md`
    *   Plan: `conductor/track22/plan.md`
    *   Audit2 findings: Functional findings 5, 6, 7, remaining source/doc/test layout gaps
    *   Key additions: scan diff-summary integration, symbol persistence/storage seams, remaining planned modules or documented shims, missing docs artifacts, black-box CLI coverage, `cargo deny`

## Milestone E: Historical Intelligence Tracks (Completed)

*   **Track E4-4: Runtime Usage in Risk Scoring**
    *   Status: Completed
    *   Spec: `conductor/trackE4-4/spec.md`
    *   Plan: `conductor/trackE4-4/plan.md`
    *   Goal: Wire the extracted `runtime_usage` data into the risk scoring engine and verification predictor.
    *   Key additions: `runtime_usage_delta` to `ImpactPacket`, env-var new dependency risk weight, config-key changes risk weight, runtime predictions.

## Milestone F: Predictive Verification Tracks (Completed)

## Milestone G: IDE Integration Tracks (Completed)

## Milestone H: Cross-Repo and Narrative Reporting (Completed)

## Completed Tracks (Phase 2 - Partial/Needs Remediation)

*   **Track 29: Advanced Narrative Reporting (Gemini)** (Status: Partial)
*   **Track 28: Federated Intelligence (Cross-Repo)** (Status: Partial)
*   **Track 27: LSP-Lite ChangeGuard Daemon** (Status: Fail)
*   **Track 26: Predictive Verification (Dependency-Aware)** (Status: Fail/Partial)
*   **Track 25: Hotspot Identification (Risk Density)** (Status: Partial)
*   **Track 24: Complexity Indexing (Spike & Implementation)** (Status: Partial Pass)
*   **Track 23: Temporal Intelligence (History Extraction)** (Status: Partial Pass)

*   **Track 13: Final Integration** (Status: Completed with follow-up required)
*   **Track 12: UI/UX Refinement** (Status: Completed)
*   **Track 11: Ask Gemini Baseline** (Status: Completed)
*   **Track 10: State SQLite Persistence** (Status: Completed)
*   **Track 9: Change Risk Analysis Engine** (Status: Completed)
*   **Track 8: Determinism Contract and Subprocess Control** (Status: Completed)
*   **Track 7: Language-Aware Symbol Extraction** (Status: Completed)
*   **Track 6: Watch Mode and Batch Debouncing** (Status: Completed)
*   **Track 5: Basic Impact Packet Shell** (Status: Completed)
*   **Track 4: Git Scan Foundation** (Status: Completed)
*   **Track 3: Config and Rule Loading** (Status: Completed)
*   **Track 2: Doctor and Platform Detection** (Status: Completed)
*   **Track 1: Repo-Local State Layout and Init** (Status: Completed)
*   **Track 0: Bootstrap CLI Skeleton** (Status: Completed)

## Milestone R: System Architecture Refactoring (Active)

*   **Track R1-1: Impact Orchestrator Extraction & Decomposition**
    *   Status: Active
    *   Spec: `conductor/trackR1-1/spec.md`
    *   Plan: `conductor/trackR1-1/plan.md`
    *   Goal: Decompose monolithic `impact.rs` into `ImpactOrchestrator` and modular enrichment providers in `src/impact/enrichment/`.
    *   Key additions: `src/impact/orchestrator.rs`, `src/impact/enrichment/mod.rs`, `src/impact/enrichment/api.rs`, etc.

## Workflow

1.  **Plan**: `@architecture-planner` creates `conductor/trackN/plan.md`.
2.  **Push Plan**: Commit and push plan to `main`.
3.  **Implement**: `@generalist` (Implementer) creates a new branch and works on the task.
4.  **Review**: `@rust-triage-specialist` or `@frontend-reviewer` (Reviewer) audits the branch.
5.  **Iteration**: If review fails, Implementer fixes.
6.  **Merge**: If review passes, create PR or merge into `main`.
7.  **Next**: Start next track.

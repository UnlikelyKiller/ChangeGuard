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


## Milestone KG: Knowledge Graph & Semantic Intelligence (Completed)

*   **Track G1: CozoDB Integration & Schema**
    *   Status: Completed
    *   Spec: `conductor/trackG1/spec.md`
    *   Plan: `conductor/trackG1/plan.md`
    *   Goal: Implement Phase 1 of Knowledge Graph: Add CozoDB engine and define Datalog relations.
    *   Key additions: `cozo` crate, `src/state/storage/cozo.rs`, `:create node/edge` relations.

*   **Track G2: Unified Ledger Schema**
    *   Status: Completed
    *   Spec: `conductor/trackG2/spec.md`
    *   Plan: `conductor/trackG2/plan.md`
    *   Goal: Mirror existing SQLite data into CozoDB and implement the migration bridge.
    *   Key additions: `src/state/migration/cozo_port.rs`, mirrored `ledger_entry`/`project_symbol` relations.

*   **Track G3: Graph Ingestion Engine**
    *   Status: Completed
    *   Spec: `conductor/trackG3/spec.md`
    *   Plan: `conductor/trackG3/plan.md`
    *   Goal: Native parser for `graph.json` with semantic-to-ledger provenance matching.
    *   Key additions: `src/index/graph_loader.rs`, batch graph loading.

*   **Track G4: Semantic Impact Enrichment**
    *   Status: Completed
    *   Spec: `conductor/trackG4/spec.md`
    *   Plan: `conductor/trackG4/plan.md`
    *   Goal: Implement Datalog reachability queries and integrate KG enrichment into `ImpactOrchestrator`.
    *   Key additions: `src/impact/enrichment/kg_provider.rs`, semantic neighbor queries.

*   **Track G5: Visual Intelligence & Navigation**
    *   Status: Completed
    *   Spec: `conductor/trackG5/spec.md`
    *   Plan: `conductor/trackG5/plan.md`
    *   Goal: Native Rust `viz` command for HTML graph export with heatmap support.
    *   Key additions: `src/commands/viz.rs`, interactive HTML templates.

*   **Track G6: Native Structural Extraction (De-coupling Part 1)**
    *   Status: Completed
    *   Spec: `conductor/trackG6/spec.md`
    *   Plan: `conductor/trackG6/plan.md`
    *   Goal: Port AST link discovery from Python to native Rust `tree-sitter`.
    *   Key additions: `src/languages/rust.rs`, `src/index/mod.rs` (LinkResolver).

*   **Track G7: Native Semantic Extraction (De-coupling Part 2)**
    *   Status: Completed
    *   Spec: `conductor/trackG7/spec.md`
    *   Plan: `conductor/trackG7/plan.md`
    *   Goal: Complete standalone independence by implementing native semantic extraction.
    *   Key additions: `src/ai/semantic_extractor.rs`, removal of `graphifyy` dependency.


## Milestone M7: Engineering Coverage Deepening (Completed)

*   **Track M7-1: Trace Config & SDK Dependency Detection**
    *   Status: Completed
    *   Spec: `conductor/trackM7-1/spec.md`
    *   Plan: `conductor/trackM7-1/plan.md`
    *   Goal: Detect observability pipeline config changes (otel-collector, Jaeger, DataDog, Grafana Agent) and third-party SDK import additions (Stripe, Auth0, Twilio, etc.).
    *   Key Additions: `src/coverage/traces.rs`, `src/coverage/sdk.rs`, Go support in `src/index/references.rs`, `CoverageConfig` in `src/config/model.rs`.

*   **Track M7-2: Service-Map Derivation**
    *   Status: Completed
    *   Spec: `docs/observability-plan2.md` §5
    *   Plan: `conductor/trackM7-2/plan.md`
    *   Goal: Infer service boundaries from route/handler/data-model topology, derive cross-service dependency edges.
    *   Key Additions: `src/coverage/services.rs`, migration M15, `service_map_delta` in `ImpactPacket`, Go support in extension lists.

*   **Track M7-3: Data-Flow Coupling Risk**
    *   Status: Completed
    *   Dependencies: M7-2
    *   Spec: `conductor/trackM7-3/spec.md`
    *   Plan: `conductor/trackM7-3/plan.md`
    *   Goal: Flag call chains where route handlers and their data models co-change, detect incomplete refactors.
    *   Key additions: `src/coverage/dataflow.rs`, `DataFlowMatch` type, cycle detection, change-percentage threshold, SQL table-name fallback.

*   **Track M7-4: Deployment Manifest Awareness**
    *   Status: Completed
    *   Spec: `conductor/trackM7-4/spec.md`
    *   Plan: `conductor/trackM7-4/plan.md`
    *   Goal: Classify Dockerfile, docker-compose, k8s, terraform, and helm changes with tiered risk weighting.
    *   Key additions: `src/coverage/deploy.rs`, `ManifestType` enum, `DeployManifestChange` type, Dockerfile COPY/ADD scanning.

*   **Track M7-5: CI Pipeline Self-Awareness**
    *   Status: Completed
    *   Spec: `conductor/trackM7-5/spec.md`
    *   Plan: `conductor/trackM7-5/plan.md`
    *   Goal: Surface risk when CI config itself changes in a diff, detect CI+source co-change patterns.
    *   Key additions: Extend `src/index/ci_gates.rs`, pre-commit hook awareness.

*   **Track M7-6: ADR Staleness Detection**
    *   Status: Completed
    *   Spec: `conductor/trackM7-6/spec.md`
    *   Plan: `conductor/trackM7-6/plan.md`
    *   Goal: Flag retrieved ADRs exceeding configurable age threshold with severity tiers and recently-updated exemption.
    *   Key additions: Extend `RelevantDecision` with `staleness_days`, multi-source age detection.

*   **Track M7-7: Impact Packet Extension & Enrichment Integration**
    *   Status: Completed
    *   Dependencies: M7-1..M7-6
    *   Spec: `docs/observability-plan2.md` §10
    *   Plan: `conductor/trackM7-7/plan.md`
    *   Goal: Wire all M7 detection into ImpactPacket, risk scoring, human output, and ask context. Master kill switch via `[coverage].enabled`.
    *   Key additions: 5 new ImpactPacket fields, 7 enrichment hooks, 7 human output sections, per-dimension kill switches.


## Milestone R: System Architecture Refactoring (Completed)

*   **Track R1-1: Impact Orchestrator Extraction & Decomposition**
    *   Status: Completed
    *   Spec: `conductor/trackR1-1/spec.md`
    *   Plan: `conductor/trackR1-1/plan.md`
    *   Goal: Decompose monolithic `impact.rs` into `ImpactOrchestrator` and modular enrichment providers in `src/impact/enrichment/`.

*   **Track R1-2: Monolithic Analysis Decomposition**
    *   Status: Completed
    *   Spec: `conductor/trackR1-2/spec.md`
    *   Plan: `conductor/trackR1-2/plan.md`
    *   Goal: Decompose the ~3,000 line `src/impact/analysis.rs` into a registry of modular `RiskProvider` implementations.

*   **Track R1-3: State Migrations Decomposition**
    *   Status: Completed
    *   Spec: `conductor/trackR1-3/spec.md`
    *   Plan: `conductor/trackR1-3/plan.md`
    *   Goal: Decompose monolithic `migrations.rs` into a modular `src/state/migrations/` directory.

*   **Track R1-4: Project Index Decomposition**
    *   Status: Completed
    *   Spec: `conductor/trackR1-4/spec.md`
    *   Plan: `conductor/trackR1-4/plan.md`
    *   Goal: Decompose monolithic `project_index.rs` into orchestrator and modular workers.


## Milestone S: Global Intelligence & Precision Search (Completed)

*   **Track S1: High-Performance Global Code Search**
    *   Status: Completed
    *   Spec: `conductor/trackS1/spec.md`
    *   Plan: `conductor/trackS1/plan.md`
    *   Goal: Implement sub-millisecond trigram-based regex search across the federated codebase.
    *   Key additions: Tantivy engine, Trigram pre-filtering, streaming indexer, UTF-8 normalization.

*   **Track S2: Precise LSP-Based Indexing (SCIP/LSIF)**
    *   Status: Completed
    *   Spec: `conductor/trackS2/spec.md`
    *   Plan: `conductor/trackS2/plan.md`
    *   Goal: Ingest SCIP indices for compiler-grade precision in navigation and impact analysis.
    *   Key additions: SCIP Protobuf ingestion, symbol mapping, stale detection, `--scip` flag.

*   **Track S3: Semantic Discovery for "Code Snippets"**
    *   Status: Completed
    *   Spec: `conductor/trackS3/spec.md`
    *   Plan: `conductor/trackS3/plan.md`
    *   Goal: Implement local vector embedding and search for fine-grained code logic blocks.
    *   Key additions: Tree-sitter AST chunking, local vector embeddings, CozoDB HNSW search, `ask --semantic`.


## Milestone T: Predictable Verification (Planning)

*   **Track T1: Predictive CI Gate Analysis & Failure Explanation**
    *   Status: Completed
    *   Spec: `conductor/trackT1/spec.md`
    *   Plan: `conductor/trackT1/plan.md`
    *   Goal: Predict CI gate failures and provide failure explanations using historical data and local LLM.
    *   Key additions: `ci_outcome_history` table, `ExplanationEngine`, `--explain` flag for verify.

*   **Track T2: Probabilistic Verification Reordering**
    *   Status: Completed
    *   Spec: `conductor/trackT2/spec.md`
    *   Plan: `conductor/trackT2/plan.md`
    *   Goal: Reorder verification execution to minimize time to first failure based on historical probability.
    *   Key additions: `src/verify/probability.rs`, verification test reordering logic.

## Milestone D: Documentation & Deep Intelligence (In Planning)

*   **Track 50-1: Document Template Engine & Basic Exports**
    *   Status: Completed
    *   Spec: `conductor/track50-1/spec.md`
    *   Plan: `conductor/track50-1/plan.md`
    *   Goal: Create a system for querying the CozoDB KG and exporting structural data to Markdown/Mermaid formats.
    *   Key additions: `src/docs/generator.rs`, `changeguard index --export-docs`.

*   **Track 50-2: Advanced Passive Doc Types (13+ formats)**
    *   Status: Completed
    *   Spec: `conductor/track50-2/spec.md`
    *   Plan: `conductor/track50-2/plan.md`
    *   Goal: Implement specialized documentation types (Module maps, Service boundaries, Dependency health scoring).
    *   Key additions: Datalog query templates for 13+ documentation types.

*   **Track 51-1: Probabilistic Reachability & Dead Code Detection**
    *   Status: Completed
    *   Spec: `conductor/track51-1/spec.md`
    *   Plan: `conductor/track51-1/plan.md`
    *   Goal: Implement confidence-based dead code detection by blending Graph reachability with Git activity and test history.
    *   Key additions: `src/impact/analysis/dead_code.rs`, `ConfidenceScorer`.

*   **Track 52-1: Real-time Graph Sync (Watcher Bridge)**
    *   Status: Completed
    *   Spec: `conductor/track52-1/spec.md`
    *   Plan: `conductor/track52-1/plan.md`
    *   Goal: Extend the `watch` command to perform incremental AST parsing and update the CozoDB Knowledge Graph in real-time.
    *   Key additions: `src/index/incremental.rs`, watcher-to-graph sync logic.

*   **Track 52-2: Live Viz (Arc Diagram & WebSocket Server)**
    *   Status: Completed
    *   Spec: `conductor/track52-2/spec.md`
    *   Plan: `conductor/track52-2/plan.md`
    *   Goal: Build a local WebSocket server to push real-time graph deltas to an interactive Arc Diagram visualization.
    *   Key additions: `src/commands/viz_server.rs`, D3.js Arc Diagram template.

*   **Track 53-1: Storage Infrastructure Stabilization (CozoDB Fork Integration)**
    *   Status: Completed
    *   Spec: `conductor/track53-1/spec.md`
    *   Plan: `conductor/track53-1/plan.md`
    *   Goal: Stabilize the Knowledge Graph storage by migrating to a dedicated CozoDB fork and resolving platform-specific path-handling and query concurrency issues.
    *   Key additions: `UnlikelyKiller/cozo-redux` dependency (Git), `sled` backend centralization, parameterized Cozo scripts, UTF-8 path normalization in `IncrementalSyncEngine`, 384-dimension runtime fallback.

*   **Track 54-1: Native Code-Aware Tokenization (Tree-Sitter FTS Integration)**
    *   Status: Completed
    *   Spec: `conductor/track54-1/spec.md`
    *   Plan: `conductor/track54-1/plan.md`
    *   Goal: Replace generic FTS tokenizers with a native Tree-Sitter implementation in CozoDB to improve search precision for code symbols, macros, and structured comments.
    *   Key additions: `tree-sitter` integration in `cozo-redux` FTS, language-aware tokenization rules, `index --fts-mode code` flag.

*   **Track 55-1: Maintenance & Migration (Update Command)**
    *   Status: In Planning
    *   Spec: `conductor/track55-1/spec.md`
    *   Plan: `conductor/track55-1/plan.md`
    *   Goal: Implement an `update` command to handle repository-level state migration, schema upgrades, and optional binary self-updating.
    *   Key additions: `update --migrate` (re-index state), `update --binary` (cargo install), and automated health checks.

## Milestone V: Semantic Search Restoration (In Planning)

*   **Track 56-1: Restore Native Semantic Search Path**
    *   Status: Completed
    *   Spec: `conductor/track56-1/spec.md`
    *   Plan: `conductor/track56-1/plan.md`
    *   Goal: Re-enable the HNSW index for `snippet_embedding` and route the fallback through native cozo-redux distance ops (`cos_dist`, `l2_dist`). Resolves `docs/help2.md`.
    *   Key additions: HNSW index restoration in `vector_store.rs`, Cozo-native `cos_dist` fallback, `CozoStorage::get_indices()`, regression tests in `tests/cozo_vector_ops.rs` and `tests/semantic_search.rs`.

## Milestone H: Dependency Hygiene (In Planning)

*   **Track 57-1: Dependency Alert Remediation**
    *   Status: Completed
    *   Spec: `conductor/track57-1/spec.md`
    *   Plan: `conductor/track57-1/plan.md`
    *   Goal: Resolve ChangeGuard's `tantivy -> lru` Dependabot alert and consume the CozoDB-redux-owned `swapvec -> lz4_flex` remediation.
    *   Key additions: Dependency compatibility matrix, Tantivy upgrade, CozoDB-redux `6690fdac` lockfile update, ChangeGuard skill guidance, and verification evidence.

## Milestone B: AI-Brains Integration (Completed)

*   **Track B1: BridgeRecord Data Model & Schema**
    *   Status: Completed
    *   Spec: `conductor/trackB1/spec.md`
    *   Plan: `conductor/trackB1/plan.md`
    *   Goal: Implement the foundational BridgeRecord data model (v0.2) for NDJSON-based communication.
    *   Key additions: `src/bridge/model.rs`, NDJSON serialization/deserialization logic.

*   **Track B2: bridge export Command**
    *   Status: Completed
    *   Spec: `conductor/trackB2/spec.md`
    *   Plan: `conductor/trackB2/plan.md`
    *   Goal: Implement `changeguard bridge export` to emit hotspots and ledger deltas as NDJSON.
    *   Key additions: `src/bridge/export.rs`, `bridge export` CLI subcommand.

*   **Track B3: bridge import Command & Impact Enrichment**
    *   Status: Completed
    *   Spec: `conductor/trackB3/spec.md`
    *   Plan: `conductor/trackB3/plan.md`
    *   Goal: Implement `changeguard bridge import` and enrich ImpactPacket with AI-Brains insights.
    *   Key additions: `src/bridge/import.rs`, `ImpactPacket` enrichment, `bridge import` CLI subcommand.

*   **Track B4: bridge query Command**
    *   Status: Completed
    *   Spec: `conductor/trackB4/spec.md`
    *   Plan: `conductor/trackB4/plan.md`
    *   Goal: Implement `changeguard bridge query` with shell execution fallback for AI-Brains recall.
    *   Key additions: `src/bridge/client.rs`, `bridge query` CLI subcommand.

*   **Track B5: Named Pipe IPC Integration**
    *   Status: Completed
    *   Spec: `conductor/trackB5/spec.md`
    *   Plan: `conductor/trackB5/plan.md`
    *   Goal: Implement synchronous Windows Named Pipe client for real-time communication with ai-brainsd.
    *   Key additions: `src/bridge/ipc.rs`, hang-protected IPC connection logic.

*   **Track B6: Unified Retrieval in Ask**
    *   Status: Completed
    *   Spec: `conductor/trackB6/spec.md`
    *   Plan: `conductor/trackB6/plan.md`
    *   Goal: Integrate AI-Brains memories into the `changeguard ask` context assembly.
    *   Key additions: Dual-retrieval logic in `ask` command, context injection.

*   **Track B7: Verification Feedback Loop**
    *   Status: Completed
    *   Spec: `conductor/trackB7/spec.md`
    *   Plan: `conductor/trackB7/plan.md`
    *   Goal: Push verification outcomes to AI-Brains via IPC.
    *   Key additions: `src/bridge/notify.rs`, post-verification hooks.

*   **Track R-B: Milestone B Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackRB/spec.md`
    *   Plan: `conductor/trackRB/plan.md`
    *   Goal: Address Codex findings: hang protection, thread leak prevention, and completed dual-retrieval.
    *   Key additions: Process timeouts, schema validation, decoupled DTOs.

*   **Track R-B2: Master Remediation & Hardening**
    *   Status: In Progress
    *   Spec: `conductor/trackRB2/spec.md`
    *   Plan: `conductor/trackRB2/plan.md`
    *   Goal: Address Master Review findings: real process killing, non-blocking IPC, strict schema enforcement, and deduplicated ask context.
    *   Key additions: `Child::kill` logic, non-blocking pipe reads, strict version gating, unified prompt assembly.

## Workflow

1.  **Plan**: `@architecture-planner` creates `conductor/trackN/plan.md`.
2.  **Push Plan**: Commit and push plan to `main`.
3.  **Implement**: `@generalist` (Implementer) creates a new branch and works on the task.
4.  **Review**: `@rust-triage-specialist` or `@frontend-reviewer` (Reviewer) audits the branch.
5.  **Iteration**: If review fails, Implementer fixes.
6.  **Merge**: If review passes, create PR or merge into `main`.
7.  **Next**: Start next track.

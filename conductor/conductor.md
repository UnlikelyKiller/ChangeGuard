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


## Milestone T: Predictable Verification (Completed)

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

## Milestone D: Documentation & Deep Intelligence (Completed)

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
    *   Status: Completed
    *   Spec: `conductor/track55-1/spec.md`
    *   Plan: `conductor/track55-1/plan.md`
    *   Goal: Implement an `update` command to handle repository-level state migration, schema upgrades, and optional binary self-updating.
    *   Key additions: `update --migrate` (re-index state), `update --binary` (cargo install), and automated health checks.

## Milestone V: Semantic Search Restoration (Completed)

*   **Track 56-1: Restore Native Semantic Search Path**
    *   Status: Completed
    *   Spec: `conductor/track56-1/spec.md`
    *   Plan: `conductor/track56-1/plan.md`
    *   Goal: Re-enable the HNSW index for `snippet_embedding` and route the fallback through native cozo-redux distance ops (`cos_dist`, `l2_dist`). Resolves `docs/help2.md`.
    *   Key additions: HNSW index restoration in `vector_store.rs`, Cozo-native `cos_dist` fallback, `CozoStorage::get_indices()`, regression tests in `tests/cozo_vector_ops.rs` and `tests/semantic_search.rs`.

## Milestone H: Dependency Hygiene (Completed)

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
    *   Status: Completed
    *   Spec: `conductor/trackRB2/spec.md`
    *   Plan: `conductor/trackRB2/plan.md`
    *   Goal: Address Master Review findings: real process killing, non-blocking IPC, strict schema enforcement, and deduplicated ask context.
    *   Key additions: `Child::kill` logic, non-blocking pipe reads, strict version gating, unified prompt assembly.

*   **Track R-B3: Audit Remediation & Spec Alignment**
    *   Status: Completed
    *   Spec: `conductor/trackRB3/spec.md`
    *   Plan: `conductor/trackRB3/plan.md`
    *   Goal: Address findings from `integration-audit.md`: spec-compliant schema, robust IPC protocol, CLI filtering, and path abstractions.
    *   Key additions: Full metadata BridgeRecord, newline-delimited framing, selective export, Layout-based paths.

## Milestone C: True Unification (Completed)

Cross-repo phase with AI-Brains Phase 18. Transforms IPC bridge from passive transport into active orchestration.

- [x] C1: Contextual Risk Export & Structured MADR Fields
- [x] C2: AI-Brains Domain Schema & Cross-Domain Reachability
- [x] C3: Predictive Verification IPC & Watcher Intervention

*   **Track C1: Contextual Risk Export & Structured MADR Fields**
    *   Status: Completed
    *   Spec: `conductor/trackC1/spec.md`
    *   Plan: `conductor/trackC1/plan.md`
    *   Goal: Add scope-based hotspot export and structured MADR field emission (not pre-formatted markdown) for AI-Brains nightly ingestion.
    *   Key additions: `--scope <paths>` filtering in `bridge export --hotspots`, `--madr` flag for structured decision fields.

*   **Track C2: AI-Brains Domain Schema & Cross-Domain Reachability**
    *   Status: Completed
    *   Dependencies: C1
    *   Spec: `conductor/trackC2/spec.md`
    *   Plan: `conductor/trackC2/plan.md`
    *   Goal: Add AI-Brains domain relations (Turn, Session, Memory, Decision) to CozoDB and define Datalog rules for cross-domain reachability (conversation→AST and AST→conversation).
    *   Key additions: 4 new CozoDB relations in `src/state/storage_cozo.rs`, 6 cross-domain query methods, 14 new tests.

*   **Track C3: Predictive Verification IPC & Watcher Intervention**
    *   Status: Completed
    *   Dependencies: C1, C2
    *   Spec: `conductor/trackC3/spec.md`
    *   Plan: `conductor/trackC3/plan.md`
    *   Goal: Expose predictive verification via IPC for AI-Brains capture gate; extend watcher to emit risk alerts on high temporal coupling.
    *   Key additions: `ipc_verify.rs` module, `BridgePayload::RiskAlert`, watcher integration.

## Milestone I: Issue Remediation (Completed)

Systematic fixes from the 2026-05-20 comprehensive command audit (`docs/issues.md`). Organized in four phases matching the audit's priority tiers. CG-2 (AI-Brains FTS5 query escaping) is excluded — it will be remediated in the ai-brains repository.

### Phase 1 — Hotfixes

*   **Track I1-1: Local Model URL Hardening & Error Transparency**
    *   Status: Completed
    *   Spec: `conductor/trackI1-1/spec.md`
    *   Plan: `conductor/trackI1-1/plan.md`
    *   Issues: CG-1a, CG-1b
    *   Goal: Change the default `local_model.base_url` from `localhost` to `127.0.0.1` to avoid Windows IPv6 resolution; surface the inner `ureq::Error::Transport` cause instead of swallowing it.
    *   Key files: `src/local_model/client.rs`, `src/config/model.rs` (default URL), `src/config/defaults.rs`

*   **Track I1-2: Self-Federation False Positive Exclusion**
    *   Status: Completed
    *   Spec: `conductor/trackI1-2/spec.md`
    *   Plan: `conductor/trackI1-2/plan.md`
    *   Issues: CG-3
    *   Goal: Prevent `check_cross_repo_impact` from treating the current repository as an invalid sibling, eliminating the spurious "medium" risk elevation. Also align `impact.rs` schema path lookup with the current `.changeguard/state/schema.json` location (it currently only checks the legacy `.changeguard/schema.json`).
    *   Key files: `src/federated/impact.rs`, `src/federated/storage.rs`

*   **Track I1-3: Log Verbosity Default Filter**
    *   Status: Completed
    *   Spec: `conductor/trackI1-3/spec.md`
    *   Plan: `conductor/trackI1-3/plan.md`
    *   Issues: CG-5
    *   Goal: Set a structured default `EnvFilter` in `tracing-subscriber` init that silences `graph_builder`, `tantivy`, and `sled` at `warn`, while preserving `RUST_LOG` override and adding a `--verbose` flag that restores full `info` output.
    *   Key files: `src/main.rs`, `src/commands/*.rs` (CLI flag wiring)

### Phase 2 — Reliability

*   **Track I2-1: Stale Index Warning Banner**
    *   Status: Completed
    *   Spec: `conductor/trackI2-1/spec.md`
    *   Plan: `conductor/trackI2-1/plan.md`
    *   Issues: CG-4
    *   Goal: Emit a yellow `[WARN]` staleness banner on `search`, `ask`, `dead-code`, and `hotspots` when the index exceeds a configurable `stale_threshold_days` (default 3). Add optional `--auto-index` flag to `search` and `ask`.
    *   Key files: `src/commands/search.rs`, `src/commands/ask.rs`, `src/commands/dead_code.rs`, `src/commands/hotspots.rs`, `src/config/model.rs`

*   **Track I2-2: Read-Only Storage Init Fast-Path**
    *   Status: Completed
    *   Spec: `conductor/trackI2-2/spec.md`
    *   Plan: `conductor/trackI2-2/plan.md`
    *   Issues: CG-6
    *   Goal: Skip full migration verification (SQLite WAL setup + CozoDB schema check) for known read-only commands (`search`, `hotspots`, `ledger status`, `config verify`). Target: eliminate the 500ms–1s cold-start overhead on those paths.
    *   Key files: `src/state/storage.rs`, `src/state/storage_cozo.rs`, `src/main.rs`

*   **Track I2-3: Agent Dotfile Exclusion**
    *   Status: Completed
    *   Spec: `conductor/trackI2-3/spec.md`
    *   Plan: `conductor/trackI2-3/plan.md`
    *   Issues: CG-7
    *   Goal: Add `.claude`, `.codex`, `.opencode/**` to the default `ignore_patterns` so agent config files are never flagged as "unsupported language" in scan/impact output.
    *   Key files: `src/config/defaults.rs` (`DEFAULT_CONFIG` ignore_patterns), `src/index/mod.rs` (hardcoded exclusion list if present)

*   **Track I2-4: Doctor Completions Endpoint Ping**
    *   Status: Completed
    *   Spec: `conductor/trackI2-4/spec.md`
    *   Plan: `conductor/trackI2-4/plan.md`
    *   Issues: CG-10
    *   Goal: Add a `POST /v1/chat/completions` liveness probe to `doctor`, separate from the existing embeddings ping. Report two distinct lines: `Embedding model` and `Completion model`. Emit yellow if completions are unreachable while embeddings succeed.
    *   Key files: `src/commands/doctor.rs`, `src/local_model/client.rs`

### Phase 3 — Feature Depth

*   **Track I3-1: Audit Command Enrichment**
    *   Status: Completed
    *   Spec: `conductor/trackI3-1/spec.md`
    *   Plan: `conductor/trackI3-1/plan.md`
    *   Issues: CG-8
    *   Goal: Expand `changeguard audit` from pending-only output to a multi-section health report: commit velocity (last 30 days), top churned files, `ci_outcome_history` pass/fail trend, oldest unupdated ADR, hotspot delta since last audit, unaudited drift summary. Add `--json` flag.
    *   Key files: `src/commands/audit.rs`, `src/ledger/db.rs`, `src/state/storage.rs`

*   **Track I3-2: Hotspot Score Log-Scaling**
    *   Status: Completed
    *   Spec: `conductor/trackI3-2/spec.md`
    *   Plan: `conductor/trackI3-2/plan.md`
    *   Issues: CG-9
    *   Goal: Apply `log1p` normalization to the raw `complexity × frequency` product so a 22× outlier gap compresses to a readable scale. Display raw factors (`complexity`, `frequency`) as sub-columns alongside the normalized score.
    *   Key files: `src/impact/hotspots.rs`

*   **Track I3-3: Local Model Windows Preflight Check**
    *   Status: Completed
    *   Spec: `conductor/trackI3-3/spec.md`
    *   Plan: `conductor/trackI3-3/plan.md`
    *   Issues: CG-1c
    *   Goal: Add a startup connectivity check in `LocalModelClient` that (a) detects `localhost` in `base_url` on Windows and emits a warning suggesting `127.0.0.1`, (b) attempts an explicit TCP connect to `127.0.0.1` on the same port if `localhost` fails, and (c) surfaces the diagnostic in `changeguard doctor`.
    *   Key files: `src/local_model/client.rs`, `src/commands/doctor.rs`

### Phase 4 — LLM Router Hardening (Parallel)

*   **Track I4-1: VRAM Pressure Monitoring in Doctor**
    *   Status: Completed
    *   Spec: `conductor/trackI4-1/spec.md`
    *   Plan: `conductor/trackI4-1/plan.md`
    *   Goal: Surface GPU VRAM usage in `changeguard doctor` using `IDXGIAdapter3::QueryVideoMemoryInfo` (DXGI 1.4, Windows). Requires adding `windows = { version = "0.57", features = ["Win32_Graphics_Dxgi", "Win32_Graphics_Dxgi_Common"] }` to `Cargo.toml`. Emit a yellow warning when used VRAM exceeds 10.5 GB (87.5% of 12 GB B580 budget). No-op on non-Windows targets.
    *   Key files: `src/commands/doctor.rs`, `Cargo.toml`

### Phase 5 — Audit Fixes

*   **Track I5-1: Fix Regex Search Trigram Case Sensitivity**
    *   Status: Completed
    *   Spec: `conductor/trackI5-1/spec.md`
    *   Plan: `conductor/trackI5-1/plan.md`
    *   Issues: Regex search returns "No matches found" for patterns with uppercase letters
    *   Goal: Lowercase trigrams in `search_trigrams()` before creating `TermQuery` to match Tantivy's tokenization behavior.
    *   Key files: `src/search/tantivy_engine.rs`

*   **Track I5-2: Fix Scan Command Ignore Patterns Filtering**
    *   Status: Completed
    *   Spec: `conductor/trackI5-2/spec.md`
    *   Plan: `conductor/trackI5-2/plan.md`
    *   Issues: `changeguard scan` flags all dirty files including agent dotfiles
    *   Goal: Load config in `execute_scan()` and filter changes against `config.watch.ignore_patterns` using glob matching.
    *   Key files: `src/commands/scan.rs`

*   **Track I5-3: Fix viz --output Parent Directory Creation**
    *   Status: Completed
    *   Spec: `conductor/trackI5-3/spec.md`
    *   Plan: `conductor/trackI5-3/plan.md`
    *   Issues: `changeguard viz --output` fails when parent directory doesn't exist
    *   Goal: Create parent directory before writing output file.
    *   Key files: `src/commands/viz.rs`

## Milestone J: Developer Experience Hardening (Completed)

Systematic UX and reliability improvements identified in the 2026-05-20 comprehensive command audit. Each track targets a concrete friction point that degrades daily productivity.

*   **Track J1: INFO→DEBUG Log Migration**
    *   Status: Completed
    *   Spec: `conductor/trackJ1/spec.md`
    *   Plan: `conductor/trackJ1/plan.md`
    *   Goal: Move storage-init and enrichment-provider lifecycle messages from `info!` to `debug!` so common commands produce zero INFO lines on stderr. Add `[init]` tag to first-time DB creation messages.
    *   Key files: `src/state/storage_cozo.rs`, `src/state/storage.rs`, `src/impact/orchestrator.rs`, `src/impact/enrichment/*.rs`, `src/search/stream_indexer.rs`, `src/commands/search.rs`

*   **Track J2: Code-Aware Trigram Tokenizer**
    *   Status: Completed
    *   Spec: `conductor/trackJ2/spec.md`
    *   Plan: `conductor/trackJ2/plan.md`
    *   Goal: Register a `WhitespaceTokenizer + LowerCaseFilter` tokenizer (`"code_trigram"`) for the Tantivy trigrams field so cross-underscore Rust identifiers (e.g. `execute_scan`) are preserved during indexing. Fixes `search -r` returning zero results for any underscore-containing pattern.
    *   Key files: `src/search/tantivy_engine.rs`

*   **Track J3: Temporal Coupling Row Cap and Relevance Filter**
    *   Status: Completed
    *   Spec: `conductor/trackJ3/spec.md`
    *   Plan: `conductor/trackJ3/plan.md`
    *   Goal: Filter temporal coupling results to pairs involving changed files; cap output at `max_coupling_pairs` (default 50). Eliminates 500+ row explosion from agent dotfiles.
    *   Key files: `src/impact/enrichment/coupling.rs`, `src/config/model.rs`, `.changeguard/config.toml`

*   **Track J4: Global Audit Multi-Section Completion**
    *   Status: Completed
    *   Spec: `conductor/trackJ4/spec.md`
    *   Plan: `conductor/trackJ4/plan.md`
    *   Goal: Implement the five sections originally specced in I3-1 but never built: commit velocity (30d), top churned files, oldest ADR, hotspot delta, and CI trend via `verify-history.json`.
    *   Key files: `src/commands/ledger_audit.rs`, `src/commands/verify.rs`

*   **Track J5: KG Enrichment Progress Indicator and Configurable Timeout**
    *   Status: Completed
    *   Spec: `conductor/trackJ5/spec.md`
    *   Plan: `conductor/trackJ5/plan.md`
    *   Goal: Add a spinner to the KG enrichment phase and a configurable `kg_timeout_secs` (default 60). Graceful degradation on timeout so the rest of the impact report still renders.
    *   Key files: `src/impact/enrichment/kg_provider.rs`, `src/impact/orchestrator.rs`, `src/ui/spinner.rs` (new), `src/config/model.rs`

*   **Track J6: `bridge export` Stdout Default**
    *   Status: Completed
    *   Spec: `conductor/trackJ6/spec.md`
    *   Plan: `conductor/trackJ6/plan.md`
    *   Goal: Make `--out` optional; default to stdout when omitted. Enables `changeguard bridge export | jq .` and parity with `bridge verify`.
    *   Key files: `src/commands/bridge.rs`, `src/bridge/export.rs`

*   **Track J7: Dead-Code False Positive Filtering**
    *   Status: Completed
    *   Spec: `conductor/trackJ7/spec.md`
    *   Plan: `conductor/trackJ7/plan.md`
    *   Goal: Filter `#[test]` functions, `pub use` re-exports, proc/derive macros, and `extern "C"` from dead-code output. Annotate feature-gated symbols instead of flagging them as dead.
    *   Key files: `src/impact/analysis/dead_code.rs`, `src/index/entrypoint.rs`, `src/index/languages/rust.rs`


*   **Track J8: `index --check` Exit Code Fix**
    *   Status: Completed
    *   Spec: `conductor/trackJ8/spec.md`
    *   Plan: `conductor/trackJ8/plan.md`
    *   Goal: Exit 0 for stale index (usable but not current), exit 1 only for missing or corrupt. Add `--strict` flag for pipelines that require a current index.
    *   Key files: `src/commands/index.rs`

*   **Track J9: BM25 Search Snippet Output**
    *   Status: Completed
    *   Spec: `conductor/trackJ9/spec.md`
    *   Plan: `conductor/trackJ9/plan.md`
    *   Goal: Add `snippet` and `line_number` to `SearchResult`; use Tantivy's `SnippetGenerator` to show `{file}:{line}: {snippet}` for BM25 results, matching regex search output format.
    *   Key files: `src/search/tantivy_engine.rs`, `src/search/mod.rs`, `src/commands/search.rs`

*   **Track J10: `viz-server` CLI Wiring or Clean Removal**
    *   Status: Completed
    *   Spec: `conductor/trackJ10/spec.md`
    *   Plan: `conductor/trackJ10/plan.md`
    *   Goal: Wire the `viz-server` subcommand into CLI dispatch (if implementation is functional) or delete the dead `src/commands/viz_server.rs` file (if it is a stub). Either path eliminates the "unrecognized subcommand" error.
    *   Key files: `src/cli.rs`, `src/commands/mod.rs`, `src/commands/viz_server.rs`


## Milestone K: Service Discovery & Storage Hardening (Completed)

*   **Track K1: Storage Resilience (Windows & Vector Integrity)**
    *   Status: Completed
    *   Spec: `conductor/trackK1/spec.md`
    *   Plan: `conductor/trackK1/plan.md`
    *   Goal: Fix Tantivy segment persistence on Windows via explicit commit verification; implement robust CozoDB shutdown and cold-start validation to prevent HNSW metadata corruption.
    *   Key files: `src/search/tantivy_engine.rs`, `src/state/storage_cozo.rs`, `src/commands/update.rs`

*   **Track K2: Intelligence Precision (Adaptive Inference Context)**
    *   Status: Completed
    *   Spec: `conductor/trackK2/spec.md`
    *   Plan: `conductor/trackK2/plan.md`
    *   Goal: Eliminate `ask --semantic` hallucinations in clean git states by pivoting to a "Codebase Oracle" mode with 90% token allocation for retrieved code chunks and mandatory source attribution.
    *   Key files: `src/commands/ask.rs`, `src/local_model/context.rs`, `src/gemini/prompt.rs`

*   **Track K3: CLI UX Polish (Proactive Recovery & Hybrid Search)**
    *   Status: Completed
    *   Spec: `conductor/trackK3/spec.md`
    *   Plan: `conductor/trackK3/plan.md`
    *   Goal: Add top-level `status` alias; implement interactive "Proactive Self-Correction" for schema mismatches; introduce heuristic search routing (Regex auto-detection) and blends.
    *   Key files: `src/cli.rs`, `src/commands/search.rs`, `src/state/storage.rs`

*   **Track K4: Service Boundary & Communication Mapping**
    *   Status: Completed
    *   Spec: `conductor/trackK4/spec.md`
    *   Plan: `conductor/trackK4/plan.md`
    *   Goal: Infer service boundaries from monorepo markers (Cargo/Npm/Go); extract inter-service communication patterns (HTTP/Graph); surface service-level blast radius in impact reports.
    *   Key files: `src/index/orchestrator.rs`, `src/coverage/services.rs` (new), `src/commands/viz.rs`

*   **Track K5: Operational Transparency (Config View & Audit Pagination)**
    *   Status: Completed
    *   Spec: `conductor/trackK5/spec.md`
    *   Plan: `conductor/trackK5/plan.md`
    *   Goal: Implement `config view` to see resolved project state; add `--limit`/`--offset` pagination to `ledger audit` for manageable transaction history.
    *   Key files: `src/commands/config.rs`, `src/commands/ledger_audit.rs`, `src/ledger/db.rs`

*   **Track K6: Temporal Risk Precision (Time-Bounded Hotspots)**
    *   Status: Completed
    *   Spec: `conductor/trackK6/spec.md`
    *   Plan: `conductor/trackK6/plan.md`
    *   Goal: Add `--commits N` and `--days N` to `hotspots` to focus analysis on recent trends; implement exponential decay weighting for hotspot scores.
    *   Key files: `src/cli.rs`, `src/impact/hotspots.rs`, `src/impact/temporal.rs`

*   **Track K7: Hotspot API Refactoring (Argument Objects)**
    *   Status: Completed
    *   Spec: `conductor/trackK7/spec.md`
    *   Plan: `conductor/trackK7/plan.md`
    *   Goal: Address signature bloat in `calculate_hotspots` by refactoring 10 positional arguments into a `HotspotQuery` struct.
    *   Key files: `src/impact/hotspots.rs`, `src/commands/hotspots.rs`

*   **Track K8: CLI Consistency (Scan Impact JSON)**
    *   Status: Completed
    *   Spec: `conductor/trackK8/spec.md`
    *   Plan: `conductor/trackK8/plan.md`
    *   Goal: Enable `--json` and `--out` support for `scan --impact` to support automated pipeline integration.
    *   Key files: `src/cli.rs`, `src/commands/scan.rs`

*   **Track K9: Unified Audit Reporting**
    *   Status: Completed
    *   Spec: `conductor/trackK9/spec.md`
    *   Plan: `conductor/trackK9/plan.md`
    *   Goal: Refactor `ledger audit` into a unified report abstraction where pagination applies holistically to ranked lists and entry tables.
    *   Key files: `src/commands/ledger_audit.rs`

*   **Track K10: Ignore-Aware Scan Cleanliness**
    *   Status: Completed
    *   Spec: `conductor/trackK10/spec.md`
    *   Plan: `conductor/trackK10/plan.md`
    *   Goal: Align `scan` and `scan --impact` with actionable Git state by filtering ignored local agent/tool directories from dirty-state and unsupported-language output.
    *   Key files: `src/commands/scan.rs`, `src/impact/orchestrator.rs`, `src/config/defaults.rs`, `src/config/model.rs`

*   **Track K11: Read-Only CozoDB Lock Resilience**
    *   Status: Completed
    *   Spec: `conductor/trackK11/spec.md`
    *   Plan: `conductor/trackK11/plan.md`
    *   Goal: Prevent concurrent read-only ChangeGuard commands from failing immediately on CozoDB locks by adding bounded wait/retry behavior and avoiding unnecessary graph opens.
    *   Key files: `src/state/storage_cozo.rs`, `src/state/storage.rs`, `src/main.rs`, `src/commands/*`

*   **Track K12: Local Model Timeout and Readiness UX**
    *   Status: Completed
    *   Spec: `conductor/trackK12/spec.md`
    *   Plan: `conductor/trackK12/plan.md`
    *   Goal: Make `doctor` and local-model `ask` readiness checks fail fast with separate embedding/completion status, split embedding/generation base URLs for the LLM2 topology, and actionable timeout diagnostics.
    *   Key files: `src/commands/doctor.rs`, `src/commands/ask.rs`, `src/local_model/client.rs`, `src/config/model.rs`

*   **Track K13: Index Freshness Recovery Workflow**
    *   Status: Completed
    *   Spec: `conductor/trackK13/spec.md`
    *   Plan: `conductor/trackK13/plan.md`
    *   Goal: Improve stale-index reporting with sample paths, machine-readable recovery guidance, and consistent auto-index support across index-dependent commands.
    *   Key files: `src/commands/index.rs`, `src/commands/search.rs`, `src/commands/ask.rs`, `src/commands/dead_code.rs`, `src/commands/hotspots.rs`

*   **Track K14: Non-Mutating Federation Export**
    *   Status: Completed
    *   Spec: `conductor/trackK14/spec.md`
    *   Plan: `conductor/trackK14/plan.md`
    *   Goal: Add a true non-mutating federation export path so users can preview schema output without writing `.changeguard/state/schema.json`.
    *   Key files: `src/commands/federate.rs`, `src/federated/schema.rs`, `src/federated/storage.rs`

*   **Track K15: Semantic Search Readiness and Fallbacks**
    *   Status: Completed
    *   Spec: `conductor/trackK15/spec.md`
    *   Plan: `conductor/trackK15/plan.md`
    *   Goal: Make `search --semantic` readiness, embedding endpoint/model/dimension mismatches, empty-result causes, and lexical fallback behavior explicit so expensive semantic queries do not fail silently.
    *   Key files: `src/commands/search.rs`, `src/semantic/*`, `src/search/tantivy_engine.rs`, `src/config/model.rs`

## Milestone O: Intent & Provenance (Tier 1) (Completed)

*   **Track O1-1: Intent Capture TUI Scaffold**
    *   Status: Completed
    *   Spec: `conductor/trackO1-1/spec.md`
    *   Plan: `conductor/trackO1-1/plan.md`
    *   Goal: Build the `ratatui` foundation for the interactive intent confirmation screen.

*   **Track O1-2: Intent Capture LLM Pipeline**
    *   Status: Completed
    *   Spec: `conductor/trackO1-2/spec.md`
    *   Plan: `conductor/trackO1-2/plan.md`
    *   Goal: Implement local Gemma 4 integration to draft intent payloads from git diffs and commit messages.

*   **Track O1-3: Git Hook Integration & UX Logic**
    *   Status: Completed
    *   Spec: `conductor/trackO1-3/spec.md`
    *   Plan: `conductor/trackO1-3/plan.md`
    *   Goal: Wire the TUI and LLM into a `commit-msg` git hook with adaptive bypass logic.

*   **Track O1-4: Heuristic Ticket Extraction**
    *   Status: Completed
    *   Spec: `conductor/trackO1-4/spec.md`
    *   Plan: `conductor/trackO1-4/plan.md`
    *   Goal: Extract Linear/Jira ticket IDs from git context to enrich the LLM prompt and TUI without brittle webhooks.

*   **Track O1-5: Cryptographic Provenance**
    *   Status: Completed
    *   Spec: `conductor/trackO1-5/spec.md`
    *   Plan: `conductor/trackO1-5/plan.md`
    *   Goal: Harden the ledger by signing every transaction with an Ed25519 developer key.

*   **Track O1-6: SOC2 Evidence Export**
    *   Status: Completed
    *   Spec: `conductor/trackO1-6/spec.md`
    *   Plan: `conductor/trackO1-6/plan.md`
    *   Goal: Provide an auditor-ready JSON/CSV export mapping ledger entries to AICPA TSP 100 controls.

*   **Track O1-R: Milestone O Remediation**
    *   Status: Completed
    *   Spec: `conductor/trackO1-R/spec.md`
    *   Plan: `conductor/trackO1-R/plan.md`
    *   Goal: Address all Critical, High, Medium, and Low findings from the GPT-5.4 Codex cross-model review of Milestone O. Key fixes: two-phase ledger write; consistent hashing; trailer preservation; TUI skip logic fixes; and full test coverage.

## Milestone RE: Refactoring Evolution (Completed)

*   **Track RE1: Decompose `src/commands/verify.rs`**
    *   Status: Completed
    *   Spec: `conductor/trackRE1/spec.md`
    *   Plan: `conductor/trackRE1/plan.md`
    *   Goal: Reduce the extreme complexity (224) of the verification engine by splitting it into specialized components.

*   **Track RE2: Modularize Monolithic Impact Analysis**
    *   Status: Completed
    *   Spec: `conductor/trackRE2/spec.md`
    *   Plan: `conductor/trackRE2/plan.md`
    *   Goal: Decompose `src/impact/analysis/mod.rs` (2,281 lines) into modular analysis providers.

*   **Track RE3: Decouple Project Index Orchestrator**
    *   Status: Completed
    *   Spec: `conductor/trackRE3/spec.md`
    *   Plan: `conductor/trackRE3/plan.md`
    *   Goal: Separate worker coordination from indexing logic in `src/index/orchestrator.rs`.

*   **Track RE4: Plugin-ize Document Generator**
    *   Status: Completed
    *   Spec: `conductor/trackRE4/spec.md`
    *   Plan: `conductor/trackRE4/plan.md`
    *   Goal: Transition `src/docs/generator.rs` to a trait-based plugin architecture for document exports.

*   **Track RE5: Segment Rust AST Parser**
    *   Status: Completed
    *   Spec: `conductor/trackRE5/spec.md`
    *   Plan: `conductor/trackRE5/plan.md`
    *   Goal: Split the monolithic `src/index/languages/rust.rs` into specialized sub-parsers by symbol type.
*   **Track RE6: Standardize CozoDB Storage Layer**
    *   Status: Completed
    *   Spec: `conductor/trackRE6/spec.md`
    *   Plan: `conductor/trackRE6/plan.md`
    *   Goal: Decouple Datalog queries and schema management from the core `CozoStorage` manager.

## Milestone H: Hardening & UX (Completed)

*   **Track H1: Semantic Engine Audit**
    *   Status: Completed
    *   Spec: `conductor/trackH1/spec.md`
    *   Plan: `conductor/trackH1/plan.md`
    *   Goal: Fix mathematical instability (NaN distances) and calibrate BM25 search scoring.
    *   Key additions: `normalize_vector` hardening in `vector_store.rs`, safe distance clamping, BM25 score calibration.

*   **Track H2: Advanced Snippet Ingestion**
    *   Status: Completed
    *   Spec: `conductor/trackH2/spec.md`
    *   Plan: `conductor/trackH2/plan.md`
    *   Goal: Transition to symbol-aware semantic indexing for improved retrieval precision.
    *   Key additions: `AstChunker` with tree-sitter integration in `chunker.rs`, symbol-level embedding metadata.

*   **Track H3: Global Knowledge Retrieval**
    *   Status: Completed
    *   Spec: `conductor/trackH3/spec.md`
    *   Plan: `conductor/trackH3/plan.md`
    *   Goal: Decouple `ask` from the impact analysis report, enabling repository-wide AI queries.
    *   Key additions: Global mode in `ask.rs` with automated Knowledge Graph context retrieval.

*   **Track H4: Windows Deployment Safety**
    *   Status: Completed
    *   Spec: `conductor/trackH4/spec.md`
    *   Plan: `conductor/trackH4/plan.md`
    *   Goal: Implement shadow-copy mechanism for `update --binary` to prevent Windows file-lock errors.
    *   Key additions: `shadow_copy_current_exe` logic in `update.rs` with automatic cleanup.

*   **Track H5: Process & Path Hardening**
    *   Status: Completed
    *   Spec: `conductor/trackH5/spec.md`
    *   Plan: `conductor/trackH5/plan.md`
    *   Goal: Fix PID management on Windows and implement encoding detection for non-UTF8 files.
    *   Key additions: PID-based process killing in `viz_server.rs`, `read_to_string_with_encoding` in `util/fs.rs`.

*   **Track H6: UX Lifecycle Polish**
    *   Status: Completed
    *   Spec: `conductor/trackH6/spec.md`
    *   Plan: `conductor/trackH6/plan.md`
    *   Goal: Support entity-based lookups for `ledger commit` and general CLI output cleanup.
    *   Key additions: Entity name resolution for transactions in `transaction.rs`.

## Milestone HP: High-Performance & Optimization (Completed)

*   **Track HP1: Fast Network Seams & Non-Blocking TCP Connect Probes**
    *   Status: Completed
    *   Spec: `conductor/trackHP1/spec.md`
    *   Plan: `conductor/trackHP1/plan.md`
    *   Goal: Accelerate model connection probes and AI-Brains integrations with non-blocking socket checks.

*   **Track HP2: Parallelized AST Chunk Ingestion & Embedding Generation**
    *   Status: Completed
    *   Spec: `conductor/trackHP2/spec.md`
    *   Plan: `conductor/trackHP2/plan.md`
    *   Goal: Parallelize Tree-Sitter parsing and embedding client queries during full indexing runs.

*   **Track HP3: Cached Vector Nodes & Incremental HNSW Appends**
    *   Status: Completed
    *   Spec: `conductor/trackHP3/spec.md`
    *   Plan: `conductor/trackHP3/plan.md`
    *   Goal: Cache vector graphs on disk to support fast, incremental append-only writes to the HNSW store.

*   **Track HP4: Snippet Ingestion Progress & HNSW Build UX**
    *   Status: Completed
    *   Spec: `conductor/trackHP4/spec.md`
    *   Plan: `conductor/trackHP4/plan.md`
    *   Goal: Build and display terminal progress bars during HNSW index construction.

## Milestone SE: SQLite Storage Migration (Completed)

*   **Track SE1: SQLite Storage Engine Migration**
    *   Status: Completed
    *   Spec: `conductor/trackSE1/spec.md`
    *   Plan: `conductor/trackSE1/plan.md`
    *   Goal: Migrate default Knowledge Graph engine to SQLite, increment version to v0.1.1, and verify incremental tests.

## Milestone CR: Codex Review Remediation (Completed)

*   **Track CR1: Incremental Semantic Indexing Deletions**
    *   Status: Completed
    *   Spec: `conductor/trackCR1/spec.md`
    *   Plan: `conductor/trackCR1/plan.md`
    *   Goal: Prune stale embeddings and file hashes during incremental indexing when a file is deleted.
    *   Key additions: `get_tracked_files`/`remove_file_hash` helpers in `SemanticDiscovery`, deletion detection in `execute_semantic_index`, path normalization for cross-platform hash matching, regression tests in `tests/semantic_search.rs`.

*   **Track CR2: Enforce Signature Verification Failures**
    *   Status: Completed
    *   Spec: `conductor/trackCR2/spec.md`
    *   Plan: `conductor/trackCR2/plan.md`
    *   Goal: Enforce that signature verification checks return a non-zero exit code on unsigned committed entries.
    *   Key additions: `all_valid = false` for UNSIGNED entries in `verify_ledger_signatures`, updated error message.

*   **Track CR3: Calibrate AI-Brains Timeout & Local Model Probe**
    *   Status: Completed
    *   Spec: `conductor/trackCR3/spec.md`
    *   Plan: `conductor/trackCR3/plan.md`
    *   Goal: Increase safety timeout margins for AI-Brains CLI fallbacks and LLM TCP preflight socket probes.
    *   Key additions: AI-Brains CLI timeout 800ms→2000ms in `client_cli.rs`; embedding and completion probe 150ms→500ms.

*   **Track CR4: Align Health Check Command Parsing**
    *   Status: Completed
    *   Spec: `conductor/trackCR4/spec.md`
    *   Plan: `conductor/trackCR4/plan.md`
    *   Goal: Unify command parsing logic between the health checker and the verification engine runner.
    *   Key additions: `extract_executable()` helper in `verify.rs` that skips `KEY=value` prefixes and strips quotes.

*   **Track CR5: CLI & Process Hardening Test Coverage**
    *   Status: Completed
    *   Spec: `conductor/trackCR5/spec.md`
    *   Plan: `conductor/trackCR5/plan.md`
    *   Goal: Add tests for verify/config CLI flags and scope force-unlock process termination safely.
    *   Key additions: `--dry-run`, `--health` (pass/fail), env-prefix regression, and `escape_cozo_string` unit tests in `tests/cli_verify.rs`.

*   **Track CR6: Strong Process Validation for Viz Server Stop**
    *   Status: Completed
    *   Spec: `conductor/trackCR6/spec.md`
    *   Plan: `conductor/trackCR6/plan.md`
    *   Goal: Validate the image name and executable path of target processes in viz-server stop before invoking taskkill.
    *   Key additions: Exact CSV image-name matching derived from `current_exe()` in `viz_server.rs` Windows stop path.

*   **Track CR7: Robust Global Ask Neighborhood Queries**
    *   Status: Completed
    *   Spec: `conductor/trackCR7/spec.md`
    *   Plan: `conductor/trackCR7/plan.md`
    *   Goal: Wire the Datalog neighborhood enrichment context to both the VectorStore and legacy chunk-pruner fallback paths in Global Ask.
    *   Key additions: `fetch_kg_neighborhood()` shared helper; neighborhood now applied after pruner fallback path in `ask.rs`.

*   **Track CR8: Escape Symbol Names in Cozo Queries**
    *   Status: Completed
    *   Spec: `conductor/trackCR8/spec.md`
    *   Plan: `conductor/trackCR8/plan.md`
    *   Goal: Escape special characters (such as single quotes) in symbol names before interpolating them into Cozo DB queries.
    *   Key additions: `pub fn escape_cozo_string()` in `ask.rs`; all Datalog symbol interpolations now escape via this helper.

*   **Track CR9: Scope Windows Shadow Copies Cleanup**
    *   Status: Completed
    *   Spec: `conductor/trackCR9/spec.md`
    *   Plan: `conductor/trackCR9/plan.md`
    *   Goal: Refine the startup sweep for *.old.*.exe files in main.rs to only clean up files belonging specifically to the current ChangeGuard binary.
    *   Key additions: Dynamic prefix derivation from `current_exe().file_stem()` in `sweep_stale_old_binaries()` in `main.rs`.

## Milestone Q: Quality & Reliability Hardening (Completed)

*   **Track Q1: Signature & Ledger Integrity**
    *   Status: Completed
    *   Spec: `conductor/trackQ1/spec.md`
    *   Plan: `conductor/trackQ1/plan.md`
    *   Goal: Investigate why 10/28 ledger entries show INVALID. Find the divergence point and fix the root cause (signing key rotation or serialization format change).
    *   Key additions: Timestamp preservation in `PendingHookTx` and `CommitRequest`, fixed signature drift in git hooks, verified hook-level roundtrip in `tests/codex_remediation.rs`.

*   **Track Q2: CLI Ergonomics & Subcommand Alignment**
    *   Status: Completed
    *   Spec: `conductor/trackQ2/spec.md`
    *   Plan: `conductor/trackQ2/plan.md`
    *   Goal: Add `version` subcommand (alias for `--version`), unify `help` vs `--help` output, and fix the `ledger audit --entity` argument parsing mismatch.
    *   Key additions: Improved CLI argument interceptor in `main.rs` (handles nested help/version), optional flag AND positional compatibility for `audit --entity`.

*   **Track Q3: Search UX & Discoverability Overhaul**
    *   Status: Completed
    *   Spec: `conductor/trackQ3/spec.md`
    *   Plan: `conductor/trackQ3/plan.md`
    *   Goal: Improve empty search results UX with guidance (suggest indexing, explain syntax). Implement auto-indexing on first use if the index is empty.
    *   Key additions: Explicit indexing status messages, guided empty-result hints (regex/index suggestions).

*   **Track Q4: Ledger API Consistency & ADR UX**
    *   Status: Completed
    *   Spec: `conductor/trackQ4/spec.md`
    *   Plan: `conductor/trackQ4/plan.md`
    *   Goal: Add `--reason` flag to `ledger rollback` for auditable state changes. Provide guidance/usage hints when `ledger adr` returns no results.
    *   Key additions: Auditable rollbacks via `EntryType::Rollback`, required `--reason` for rollback, guided ADR empty results.

*   **Track Q5: DevEx & Hook Optimization**
    *   Status: Completed
    *   Spec: `conductor/trackQ5/spec.md`
    *   Plan: `conductor/trackQ5/plan.md`
    *   Goal: Optimize the `commit-msg` hook to reduce delay and improve transparency. Cleanup the `test-entity2` artifact from the ledger if possible.
    *   Key additions: Conventional commit fast-path bypass in `commit-msg` hook (with improved risk mapping), tightened conventional prefix detection, terminal spinner for LLM drafting.


## Milestone R: Resilience & Advanced Ergonomics (Completed)

*   **Track R1: Advanced CLI Help Interceptor**
    *   Status: Completed
    *   Spec: `conductor/trackR1/spec.md`
    *   Plan: `conductor/trackR1/plan.md`
    *   Goal: Enhance the `main.rs` interceptor to detect `help <subcommand>` and transform it into `<subcommand> --help`, ensuring specific help pages are reached instead of the global menu.
    *   Key additions: Global argument pre-processor in `main.rs` that remaps `help` tokens to trailing `--help` flags.

*   **Track R2: Search Noise Reduction & Scoped Viz**
    *   Status: Completed
    *   Spec: `conductor/trackR2/spec.md`
    *   Plan: `conductor/trackR2/plan.md`
    *   Goal: Implement strict trigram pre-filtering for BM25 search in tantivy_engine.rs to reduce common-term noise. Add scoping flags (--limit, --depth, --entity) to the 'viz' command in src/commands/viz.rs.
    *   Key additions: Alphanumeric query trigram filter in `TantivySearchEngine`, recursive CozoDB reachability for scoped `viz` exports.

*   **Track R3: Proactive Index Repair & Health**
    *   Status: Completed
    *   Spec: `conductor/trackR3/spec.md`
    *   Plan: `conductor/trackR3/plan.md`
    *   Goal: Extend 'doctor' command to verify Tantivy index integrity and CozoDB graph staleness. Provide 'repair' suggestions (like 'changeguard index --full').
    *   Key additions: "Index Health" section in `doctor` report, integrity/staleness probes for search and graph indices.

*   **Track R4: Ledger Lifecycle Hardening (GC & Orphans)**
    *   Status: Completed
    *   Spec: `conductor/trackR4/spec.md`
    *   Plan: `conductor/trackR4/plan.md`
    *   Goal: Implement 'ledger gc --orphans' to identify and remove PENDING transactions older than a certain TTL or orphaned by abandoned commits.
    *   Key additions: `ledger gc` subcommand, `delete_stale_pending_transactions` query in `LedgerDb`.

*   **Track R5: Context-Aware Intelligence Defaults**
    *   Status: Completed
    *   Spec: `conductor/trackR5/spec.md`
    *   Plan: `conductor/trackR5/plan.md`
    *   Goal: In 'changeguard ask', automatically default to GLOBAL mode if no staged/dirty changes are found, instead of erroring with "No changes to analyze".
    *   Key additions: Automatic mode fallback in `execute_ask` based on `ImpactPacket` change list size.

*   **Track R6: GPU VRAM Reporting & Binary Lock Resilience**
    *   Status: Completed
    *   Spec: `conductor/trackR6/spec.md`
    *   Plan: `conductor/trackR6/plan.md`
    *   Goal: Fix the bug where `doctor` reports 0.0 GB VRAM. Investigate and mitigate Windows binary locks during `cargo install` (e.g., via a pre-install handle check).
    *   Key additions: DXGI adapter iteration to find discrete GPUs, pre-install lock check in `update` command.


## Milestone U: System Consolidation & Maintenance (Completed)

*   **Track U1: Single Integration Test Harness**
    *   Status: Completed
    *   Spec: `conductor/trackU1/spec.md`
    *   Plan: `conductor/trackU1/plan.md`
    *   Goal: Consolidate standalone integration test files into a unified test harness to mitigate Windows Application Control security blocks on multiple target binaries.

*   **Track U2: AI-Brains Daemon Status Subcommand**
    *   Status: Completed
    *   Spec: `conductor/trackU2/spec.md`
    *   Plan: `conductor/trackU2/plan.md`
    *   Goal: Add a `status` subcommand to the `ai-brains daemon` CLI to inspect if the background service is active, check bound ports, and print its PID.

*   **Track U3: Proactive SQLite PRAGMA Auditing**
    *   Status: Completed
    *   Spec: `conductor/trackU3/spec.md`
    *   Plan: `conductor/trackU3/plan.md`
    *   Goal: Audit all SQLite PRAGMA executions across both ChangeGuard and AI-Brains repositories to ensure no query-returning pragmas are called via `execute()`.


## Milestone UX: Risk Precision & Experience Hardening (Planned)

*   **Track U4: Risk De-Noising (Ignore Logic Refinement)**
    *   Status: Completed
    *   Spec: `conductor/trackU4/spec.md`
    *   Plan: `conductor/trackU4/plan.md`
    *   Goal: Refine `filter_ignored_changes` to exclude even tracked files from risk analysis if they match an ignore pattern.

*   **Track U5: Interactive Index Recovery**
    *   Status: Completed
    *   Spec: `conductor/trackU5/spec.md`
    *   Plan: `conductor/trackU5/plan.md`
    *   Goal: Prompt users to run indexing if `search --semantic` is called when the index is empty or stale.

*   **Track U6: Path-Weighted Risk Scoring**
    *   Status: Completed
    *   Spec: `conductor/trackU6/spec.md`
    *   Plan: `conductor/trackU6/plan.md`
    *   Goal: Implement risk weights for different file types (e.g., .rs vs .md) to ensure Overall Risk reflects logical impact rather than churn volume.

*   **Track U7: High-Performance Parallel Indexing**
    *   Status: Completed
    *   Spec: `conductor/trackU7/spec.md`
    *   Plan: `conductor/trackU7/plan.md`
    *   Goal: Parallelize Tree-Sitter parsing and embedding generation to reduce cold-start indexing time.

*   **Track U8: HNSW Build Speed Optimization (Incremental HNSW Appends/Caching)**
    *   Status: Completed
    *   Spec: `conductor/trackU8/spec.md`
    *   Plan: `conductor/trackU8/plan.md`
    *   Goal: Optimize vector store HNSW indexing speed by using incremental updates or caching instead of full rebuilds.
    *   Key additions: Small semantic batches now append without dropping/rebuilding HNSW; 500+ chunk batches still rebuild for stability.

*   **Track U9: Interactive Category Auto-Correction**
    *   Status: Completed
    *   Spec: `conductor/trackU9/spec.md`
    *   Plan: `conductor/trackU9/plan.md`
    *   Goal: Implement category fuzzy-matching and interactive inquire selection on invalid categories in `ledger start`.
    *   Key additions: Flexible category aliases, terminal `inquire::Select` recovery, non-TTY closest-match fallback, CLI parsing defers validation to command logic.

*   **Track U10: Flexible Local Completion Model Fallback (Cloud API / Ollama Pro)**
    *   Status: Completed
    *   Spec: `conductor/trackU10/spec.md`
    *   Plan: `conductor/trackU10/plan.md`
    *   Goal: Add fallback completion client capabilities supporting Ollama Pro Cloud API models (like minimax-m3:cloud).
    *   Key additions: `OLLAMA_CLOUD_URL`, `OLLAMA_CLOUD_API_KEY`, and `OLLAMA_CLOUD_MODEL` config resolution plus bearer-auth fallback routing in the local completion client.

*   **Track U11: Dynamic HNSW Rebuild Threshold Configuration**
    *   Status: Completed
    *   Spec: `conductor/trackU11/spec.md`
    *   Plan: `conductor/trackU11/plan.md`
    *   Goal: Make the HNSW rebuild batch threshold configurable inside `config.toml` (e.g., `semantic.hnsw_rebuild_threshold`) rather than hardcoding it, allowing performance tuning for different hardware specs.
    *   Key additions: `SemanticConfig`, default `semantic.hnsw_rebuild_threshold = 500`, validation against zero thresholds, and configurable `VectorStore` HNSW rebuild planning.

*   **Track U12: Parallel Semantic Chunks Embedding Verification**
    *   Status: Completed
    *   Spec: `conductor/trackU12/spec.md`
    *   Plan: `conductor/trackU12/plan.md`
    *   Goal: Leverage parallel requests or batched execution pipelines when retrieving embeddings for newly discovered chunks during semantic indexing to minimize overall wait times.
    *   Key additions: semantic embedding batch partitioning helper with order-preservation regression coverage for the existing rayon parallel embedding path.

*   **Track U13: Dynamic HNSW Rebuild Threshold Integration**
    *   Status: Completed
    *   Spec: `conductor/trackU13/spec.md`
    *   Plan: `conductor/trackU13/plan.md`
    *   Goal: Expose the HNSW rebuild batch threshold limit as a user-configurable parameter (e.g. `[semantic] hnsw_rebuild_threshold = 500`) in `config.toml`, enabling manual system performance tuning.
    *   Note: U11 already shipped the HNSW threshold plumbing. U13 completes the integration by adding the parallel `[semantic] concurrency` namespace field (default `None` = auto-tune) and migrating the rayon pool size out of `[local_model]`.
    *   Key additions: `concurrency: Option<usize>` on `SemanticConfig`, `semantic_concurrency()` accessor, `> 0` validation, default-TOML template entry, `format_semantic_line` reporting in `config verify`.

*   **Track U14: Semantic Indexing Concurrency Auto-Tuning**
    *   Status: Completed
    *   Spec: `conductor/trackU14/spec.md`
    *   Plan: `conductor/trackU14/plan.md`
    *   Goal: Automatically tune rayon concurrency settings during semantic index refreshes, matching logical core layouts and request-budget thresholds dynamically.
    *   Key additions: `src/semantic/concurrency.rs` (`resolve_semantic_concurrency`, `EmbedSemaphore`), stdlib `available_parallelism()`-based auto-default, separate `DEFAULT_EMBED_CAP=4` cap on concurrent embed requests, refactor of `execute_semantic_index` to consume the new helper.

*   **Track U15: Split Semantic Concurrency + Always-Visible Diagnostics**
    *   Status: Completed
    *   Spec: `conductor/trackU15/spec.md`
    *   Plan: `conductor/trackU15/plan.md`
    *   Goal: Address the U14 retrospective opportunities #5, #2, #3 in one pass: split `[semantic].concurrency` into `parse_concurrency` and `embed_concurrency` (with legacy back-compat), move the Phase 2 thread-resolution log above the empty-files early-exit, and add `--semantic-dry-run` for config diagnostics.
    *   Key additions: split fields in `SemanticConfig`, clap `Option<Option<PathBuf>>` for `--semantic-dry-run[=<path>]`, dry-run report generator using `comfy-table` (human) and `serde_json` (machine), deprecation log on legacy `concurrency` field.
    *   Dependencies: none (consolidates three U14 retrospective opportunities into one shippable unit).

*   **Track U16: Configurable Embed Concurrency Cap**
    *   Status: Completed
    *   Spec: `conductor/trackU16/spec.md`
    *   Plan: `conductor/trackU16/plan.md`
    *   Goal: Address U14 retrospective opportunity #4 — expose `DEFAULT_EMBED_CAP = 4` as `[semantic].embed_concurrency_cap` for users on non-standard hardware (beefier GPU box, Raspberry Pi, etc.).
    *   Key additions: `embed_concurrency_cap: Option<usize>` field on `SemanticConfig`, `semantic_embed_concurrency_cap()` accessor, `> 0` validation, wiring into `ResolveOptions::embed_cap` at the call site.
    *   Dependencies: U15 (so the dry-run output can show the cap alongside the other concurrency values).

*   **Track U17: Fix TOML Merge Regression for `[semantic]` Defaults**
    *   Status: Completed
    *   Spec: `conductor/trackU17/spec.md`
    *   Plan: `conductor/trackU17/plan.md`
    *   Goal: Address U14 retrospective opportunity #6 — fix the serde `#[serde(default)]` gotcha where `Option<T>` fields with non-`None` intended defaults lose those defaults when a sibling field is set in user TOML. Confirmed bug surface: `[semantic] concurrency = 4` in user config → `hnsw_rebuild_threshold` becomes `null` instead of `500`.
    *   Key additions: `default_hnsw_rebuild_threshold()` helper, `#[serde(default = "default_hnsw_rebuild_threshold")]` attribute, regression test.
    *   Dependencies: none (foundational bug fix).

*   **Track U18: Audit and Fix All `Option<T>` Serde Defaults in `Config`**
    *   Status: Completed
    *   Spec: `conductor/trackU18/spec.md`
    *   Plan: `conductor/trackU18/plan.md`
    *   Goal: Systematically apply the U17 fix to every `Option<T>` field in the config model that has a non-`None` intended default. Document the `None` defaults that are intentional.
    *   Key additions: `default_<field>()` helpers per affected field, doc comments on intentional `None` defaults, regression test per fix.
    *   Dependencies: U17 (same pattern, wider scope).

*   **Track U19: Data-Driven `config verify` Section Table**
    *   Status: Completed
    *   Spec: `conductor/trackU19/spec.md`
    *   Plan: `conductor/trackU19/plan.md`
    *   Goal: Address U14 retrospective opportunity #6 — replace the hand-wired `println!` chain in `execute_config_verify` with a data-driven `ConfigSection` trait + registry pattern. Scales to 10+ sections without per-section refactors.
    *   Key additions: `src/commands/config_verify.rs` module, `ConfigSection` trait, `ConfigRow`, `ValueSource` enum, `--json` and `--section=<name>` flags.
    *   Dependencies: U15, U16 (so the new sections can register cleanly).

*   **Track U20: Always-Visible Semantic Index Lifecycle Logging**
    *   Status: Completed
    *   Spec: `conductor/trackU20/spec.md`
    *   Plan: `conductor/trackU20/plan.md`
    *   Goal: Address U14 retrospective opportunity #2 — move the thread-resolution `info!` log to before the early-exit at `src/commands/index.rs:612`, switch the "up to date" `println!` to `tracing::info!` (per 2026 best practices: stdout = machine contract, stderr = human via tracing).
    *   Key additions: `Semantic indexing started:` lifecycle log, `Semantic indexing will process N files` phase boundary log, "up to date" via `tracing::info!`.
    *   Dependencies: none.

*   **Track U21: Non-Blocking Embed Concurrency Cap**
    *   Status: Completed
    *   Spec: `conductor/trackU21/spec.md`
    *   Plan: `conductor/trackU21/plan.md`
    *   Goal: Address U14 retrospective opportunity #5 (secondary) — replace the `parking_lot::Mutex<usize>` + `Condvar` core of `EmbedSemaphore` with an `AtomicUsize`-based non-blocking implementation. Scales better when U16's configurable cap is set higher.
    *   Key additions: `try_acquire()` (non-blocking), `try_acquire_spin(max_iters)` (bounded spin), refactored `acquire()` as a `try_acquire` loop with `std::thread::yield_now()`, `EmbedPermit::noop()` for the backoff path, 8 unit tests including race-condition stress tests.
    *   Dependencies: U16 (the cap is the load-bearing reason to do this work).

*   **Track U22: ChangeGuard LLM Query Timeout Guardrails**
    *   Status: Completed
    *   Spec: `conductor/trackU22/spec.md`
    *   Plan: `conductor/trackU22/plan.md`
    *   Goal: Introduce client-side timeout thresholds for LLM API connection queries in `changeguard ask` so that backend latency degrades gracefully with a fallback message rather than blocking pipelines indefinitely.
    *   Key additions: `--timeout <seconds>` CLI flag (default 15) on `changeguard ask`; `complete()` accepts `timeout_secs_override: Option<u64>`; new `transport_is_timeout` helper walks ureq's `io::ErrorKind::TimedOut` source chain to produce a "timed out after Ns" error message; new `AskSection` in `config verify` shows resolved timeouts; regression tests in `src/local_model/client.rs` and `tests/integration/cli_ask.rs`.
*   **Track U23: Signature Enforcement in Pre-Push Hook**
    *   Status: Completed
    *   Spec: `conductor/trackU23/spec.md`
    *   Goal: Add signature verification to `ledger status --compact --exit-code` or via a strict flag, and update pre-push hook configuration.
    *   Key additions: `--verify-signatures` flag on `ledger status`, cryptographic verification checks in status execution, updated pre-push hook template.

*   **Track U24: Reset Safety & Path Hygiene**
    *   Status: Completed
    *   Spec: `conductor/trackU24/spec.md`
    *   Goal: Add `--dry-run` and explicit pre-deletion diff to `reset`, and resolve case-mismatched path normalization.
    *   Key additions: `--dry-run` flag on `reset` command, reset plan previews, path canonicalization in discovery/setups.

*   **Track U25: Skill & Help Text Accuracy**
    *   Status: Completed
    *   Spec: `conductor/trackU25/spec.md`
    *   Goal: Fix mismatched flags, subcommands, and options in documentation, skills, and help outputs.
    *   Key additions: Corrected `--auto-index` documentation to `--incremental` in `.agents/skills/changeguard/SKILL.md` (and `.codex/` counterpart).

*   **Track U26: Verify & Audit Output Cleanup**
    *   Status: Completed
    *   Spec: `conductor/trackU26/spec.md`
    *   Goal: Suppress trailing error message on signature verification success, and clean up LLM noise in `audit --entity` output.
    *   Key additions: Count validation summary to `verify_ledger_signatures` in `src/commands/verify.rs`. Demoted fallback warnings in `src/local_model/client.rs` to `debug!` level to suppress output noise.

*   **Track U27: Ledger Subcommand Parity & GC Mode Validation**
    *   Status: Completed
    *   Spec: `conductor/trackU27/spec.md`
    *   Goal: Enforce GC modes when passing `--force` to `ledger gc`, and reconcile missing ledger subcommands.
    *   Key additions: Removed unimplemented subcommands (`ledger resume`, `ledger note`) from commands reference markdown. Changed CLI validator command parameter from `-c` to `-x` in `src/cli.rs` to resolve the duplicate `-c` flag conflict. Added explicit `--force` validation checks to `execute_ledger_gc` in `src/commands/ledger.rs`.

*   **Track U28: Init Storage Bootstrap**
    *   Status: Completed
    *   Spec: `conductor/trackU28/spec.md`
    *   Goal: Initialize the SQLite database/ledger storage directly during `init` to prevent errors on subsequent status reads.
    *   Key additions: Modified `src/commands/init.rs` to call `StorageManager::init` directly to initialize the SQLite database.

*   **Track U29: Intent Demo TTY Detection**
    *   Status: Completed
    *   Spec: `conductor/trackU29/spec.md`
    *   Goal: Detect non-interactive TTY environments in `intent demo` to prevent indefinite hangs in scripts.
    *   Key additions: Added non-interactive TTY bail out check in `src/commands/intent.rs`.

*   **Track U30: verify Warning Hygiene**
    *   Status: Completed
    *   Spec: `conductor/trackU30/spec.md`
    *   Goal: Suppress noisy empty diff warning logs on successful `verify` executions.
    *   Key additions: Demoted empty diff logs in `src/verify/semantic_predictor.rs` to `debug!` level.

*   **Track U31: Update --binary Pre-flight**
    *   Status: Completed
    *   Spec: `conductor/trackU31/spec.md`
    *   Goal: Print target installation path and add `--dry-run` to the `update --binary` command.
    *   Key additions: Added `--dry-run` to `Update` CLI command definition, mapped it in `src/cli.rs` dispatch, and implemented dry-run print warnings in `src/commands/update.rs`.


## Milestone W: Large-Repo Tracking Graph (Completed)

*   **Track W1: Entity Graph Schema and Cross-Surface Links**
    *   Status: Completed
    *   Spec: `conductor/trackW1/spec.md`
    *   Plan: `conductor/trackW1/plan.md`
    *   Goal: Build the typed graph foundation that links endpoints, handlers, symbols, tests, ADRs, ledger transactions, services, data, config, dependencies, deployments, observability, hotspots, and security boundaries.
    *   Definition of done: Deterministic schema-versioned graph relations exist, at least one impact provider consumes shared traversal helpers, migrations preserve existing state, and full ChangeGuard verification plus reinstall passes.

*   **Track W2: Public API Endpoint Ownership, Auth, and Consumer Graph**
    *   Status: Completed
    *   Dependencies: W1
    *   Spec: `conductor/trackW2/spec.md`
    *   Plan: `conductor/trackW2/plan.md`
    *   Goal: Raise endpoint tracking from 7/10 to 9/10 by linking routes, OpenAPI operations, request/response schemas, auth, services, owners, tests, and consumers.
    *   Definition of done: Endpoint output distinguishes known, inferred, configured, and unknown facts; breaking endpoint/auth/schema changes raise explicit impact risk; full verification plus reinstall passes.

*   **Track W3: ADR Lifecycle and Decision Governance**
    *   Status: Completed
    *   Dependencies: W1
    *   Spec: `conductor/trackW3/spec.md`
    *   Plan: `conductor/trackW3/plan.md`
    *   Goal: Raise ADR tracking from 6/10 to 9/10 with structured status, owner, supersession, review metadata, entity links, and impact warnings for governed code.
    *   Definition of done: ADR lifecycle is manageable through CLI commands, active and superseded decisions link to graph entities, stale/conflicting decisions affect impact, and full verification plus reinstall passes.

*   **Track W4: Service Boundary Ownership and Async Topology**
    *   Status: Completed
    *   Dependencies: W1, W2
    *   Spec: `conductor/trackW4/spec.md`
    *   Plan: `conductor/trackW4/plan.md`
    *   Goal: Raise service boundary tracking from 7/10 to 9/10 with owner overlays, runtime names, queues, topics, RPC, external calls, data stores, deploy links, and service diff output.
    *   Definition of done: Services have declared/inferred topology with conflict reporting, async boundary changes affect impact, and full verification plus reinstall passes.

*   **Track W5: Data Model and Migration Compatibility Graph**
    *   Status: Completed
    *   Dependencies: W1, W4
    *   Spec: `conductor/trackW5/spec.md`
    *   Plan: `conductor/trackW5/plan.md`
    *   Goal: Raise data model and migration tracking from 7/10 to 9/10 by classifying schema changes, ownership, compatibility risk, backfill requirements, and downstream endpoint/service/test impact.
    *   Definition of done: Migration compatibility is explicit, destructive changes raise targeted risk, data models link to services/endpoints/tests/ADRs, and full verification plus reinstall passes.

*   **Track W6: Config and Environment Variable Ownership**
    *   Status: Completed
    *   Dependencies: W1, W4
    *   Spec: `conductor/trackW6/spec.md`
    *   Plan: `conductor/trackW6/plan.md`
    *   Goal: Raise config/env tracking from 8/10 to 9/10 with requiredness, defaults, secret status, owners, environment scope, providers, rotation policy, and service-scoped diff output.
    *   Definition of done: Config output separates unknown/optional/required/defaulted states, secret values stay redacted in every output mode, config changes map to services/tests where known, and full verification plus reinstall passes.

*   **Track W7: CI/CD and Deployment Surface Ownership**
    *   Status: Completed
    *   Dependencies: W1, W4, W6
    *   Spec: `conductor/trackW7/spec.md`
    *   Plan: `conductor/trackW7/plan.md`
    *   Goal: Raise CI/CD and deployment tracking from 7/10 to 9/10 with workflow jobs, release gates, environments, artifacts, deploy manifests, owners, secrets, service links, and CI/deploy diff output.
    *   Definition of done: CI/deploy surfaces include owner/environment metadata where available, release gate weakening raises risk, manifest changes map to services and verification hints, and full verification plus reinstall passes.

*   **Track W8: Dependency, SDK, and Advisory Graph**
    *   Status: Completed
    *   Dependencies: W1, W4, W6
    *   Spec: `conductor/trackW8/spec.md`
    *   Plan: `conductor/trackW8/plan.md`
    *   Goal: Raise dependency and SDK tracking from 6/10 to 9/10 with direct/transitive package graphs, OSV-Scanner offline JSON ingestion, provider SDK links, service exposure, and vulnerability-path impact.
    *   Definition of done: Dependency paths distinguish direct/transitive edges, OSV advisory matches name evidence and affected services, no network dependency is required for baseline behavior, optional scanner imports normalize through the advisory graph, and full verification plus reinstall passes.

*   **Track W9: Test and Verification Mapping Confidence**
    *   Status: Completed
    *   Dependencies: W1, W2, W4, W5, W6
    *   Spec: `conductor/trackW9/spec.md`
    *   Plan: `conductor/trackW9/plan.md`
    *   Goal: Raise test mapping from 7/10 to 9/10 with durable test nodes, owners, risk classes, flakiness, last result, coverage confidence, and per-entity verification explanations.
    *   Definition of done: Test selection is explainable per entity, confidence/flakiness affect recommendations, missing-test gaps are surfaced for high-risk changes, and full verification plus reinstall passes.

*   **Track W10: Runtime Observability, SLO, and Alert Ownership Graph**
    *   Status: Completed
    *   Dependencies: W1, W4, W7, W9
    *   Spec: `conductor/trackW10/spec.md`
    *   Plan: `conductor/trackW10/plan.md`
    *   Goal: Raise runtime/observability tracking from 6/10 to 9/10 with metrics, logs, traces, OpenSLO reliability targets, alerts, dashboards, incidents, owners, runtime service identity, and observability coverage output.
    *   Definition of done: Observability coverage is inspectable per service/endpoint, OpenSLO objects link to services and owners, SLO and alert owner gaps are explicit, live integrations remain optional, and full verification plus reinstall passes.

*   **Track W11: Hotspot and Temporal Coupling Trends**
    *   Status: Completed
    *   Dependencies: W1, W4, W9
    *   Spec: `conductor/trackW11/spec.md`
    *   Plan: `conductor/trackW11/plan.md`
    *   Goal: Raise hotspot and temporal coupling tracking from 9/10 to 10/10 with persisted trend history, owner/service/test links, budgets, and explain output.
    *   Definition of done: Trend output is reproducible, budget violations can warn or fail by policy, hotspots include owner/service/test context when known, and full verification plus reinstall passes.

*   **Track W12: Ledger Transaction Entity Links and Validator UX**
    *   Status: Completed
    *   Dependencies: W1, W3, W9, W11
    *   Spec: `conductor/trackW12/spec.md`
    *   Plan: `conductor/trackW12/plan.md`
    *   Goal: Raise provenance/ledger tracking from 9/10 to 10/10 with validator IDs and lifecycle commands, transaction graph neighborhoods, hook diagnostics, repair commands, and stable provenance export.
    *   Definition of done: Validator UX requires no manual database edits, transaction graph output shows affected entities and evidence, hook mismatch repair is auditable, and full verification plus reinstall passes.

*   **Track W13: Security Boundary, Authz, and Policy Graph**
    *   Status: Completed
    *   Dependencies: W1, W2, W4, W6, W7, W8, W9, W12
    *   Spec: `conductor/trackW13/spec.md`
    *   Plan: `conductor/trackW13/plan.md`
    *   Goal: Raise security boundary tracking from 7/10 to 9/10 with auth/authz graph nodes, Cedar policy parsing, roles, scopes, secret dependencies, protected resources, process boundaries, and security impact output.
    *   Definition of done: Security graph output is useful without exposing secrets, Cedar principal/action/resource edges are queryable, auth/authz changes affect endpoint/service impact, protected path and process-policy changes name review requirements, and full verification plus reinstall passes.


## Milestone Y: CLI Reliability & UX Hardening (Completed)

*   **Track Y1: Integration Test Coverage for Untested Command Surfaces**
    *   Status: Completed
    *   Spec: `conductor/trackY1/spec.md`
    *   Plan: `conductor/trackY1/plan.md`
    *   Goal: Add CLI dispatch-level integration tests for 11 untested command surfaces (`config`, `endpoints`, `data-models`, `observability`, `security`, `services`, `dead-code`, `viz`, `update`, `federate`, `audit`) to prevent silent regressions.
    *   Definition of done: Each surface has ≥1 smoke test validating the full pipeline from CLI args through to output; `cargo nextest run --test integration` passes; 224 integration tests pass (13 new tests over 211 baseline).

*   **Track Y2: Standardize JSON Output Contract**
    *   Status: Completed
    *   Spec: `conductor/trackY2/spec.md`
    *   Plan: `conductor/trackY2/plan.md`
    *   Goal: Establish and enforce a project-wide JSON output contract: `--json` → stdout, `--out <file>` → file, human-readable text → stderr. Audit and fix every command surface for compliance.
    *   Definition of done: All commands with `--json` output valid JSON to stdout; all commands with `--out` write to the specified file path; human text goes to stderr; `changeguard X --json | jq` works reliably across all surfaces.

*   **Track Y3: Consolidate `scan --impact` vs Standalone `impact`**
    *   Status: Completed
    *   Spec: `conductor/trackY3/spec.md`
    *   Plan: `conductor/trackY3/plan.md`
    *   Goal: Eliminate the confusing overlap by adding `--json` and `--out` to standalone `changeguard impact`, merging internal code paths, and documenting the canonical usage.
    *   Definition of done: `changeguard impact --json` writes to stdout; `changeguard impact --out path` writes to file; internal code paths deduplicated; test coverage verifies both flags; no behavior change for `scan --impact`.

*   **Track Y4: Progress Feedback for Blocking Operations**
    *   Status: Completed
    *   Spec: `conductor/trackY4/spec.md`
    *   Plan: `conductor/trackY4/plan.md`
    *   Goal: Add spinner or status-line feedback to `changeguard ask` (15s LLM timeout), `changeguard verify` (long-running commands), `changeguard index --semantic` (local model inference), and the stale-index non-interactive guard, so users see progress during blocking operations.
    *   Definition of done: Spinner message appears before each blocking call in human mode; JSON/script modes suppress spinner; `CHANGEGUARD_NON_INTERACTIVE` env-var skips interactive prompts; tests pass.

*   **Track Y5: CLI UX Consistency — Category Enum, Dry-Run Plan, Env-Var Identity**
    *   Status: Completed
    *   Spec: `conductor/trackY5/spec.md`
    *   Plan: `conductor/trackY5/plan.md`
    *   Goal: Fix three medium-friction UX issues: (1) `ledger start --category` accepts free-string → change to `Category` enum matching `ledger atomic`; (2) `verify --dry-run` prints the verification plan instead of silent success; (3) risk analysis tracks env-var identity (not just cardinality) to catch same-cardinality replacements.
    *   Definition of done: `ledger start --category` uses enum with tab-completion; `verify --dry-run` prints full plan; risk analysis flags `DATABASE_URL → REDIS_URL` as a change even when cardinality is unchanged; all existing tests pass.

## Milestone X: Command Surface Correctness (Completed)

*   **Track X1: `ask` KG Fallback When Semantic Index Is Absent**
    *   Status: Completed
    *   Spec: `conductor/trackX1/spec.md`
    *   Plan: `conductor/trackX1/plan.md`
    *   Goal: When the semantic vector store is empty, `changeguard ask` falls back to CozoDB BM25 text search for KG context instead of returning generic LLM answers.
    *   Definition of done: `ask "query"` returns project-grounded answer using KG when no semantic index exists; fallback note printed; `--no-kg-fallback` flag suppresses it; tests pass.

*   **Track X2: `dependencies list` Populates from Cargo.lock During Index**
    *   Status: Completed
    *   Spec: `conductor/trackX2/spec.md`
    *   Plan: `conductor/trackX2/plan.md`
    *   Goal: `graph_loader.rs` parses `Cargo.lock` during `index --analyze-graph` and creates `NodeKind::Package` nodes and `DependsOn` edges so `dependencies list` works without a prior OSV audit.
    *   Definition of done: After `index --analyze-graph` on a Rust project, `dependencies list` shows 200+ packages; idempotent re-index; tests pass.

*   **Track X3: `hotspots explain` Correct Complexity and Frequency**
    *   Status: Completed
    *   Spec: `conductor/trackX3/spec.md`
    *   Plan: `conductor/trackX3/plan.md`
    *   Goal: Fix path normalization in `execute_hotspots_explain` so complexity SQL query uses the absolute path and frequency uses exact-file filtering instead of dir-prefix matching.
    *   Definition of done: `hotspots explain src/commands/hotspots.rs` shows non-zero complexity and frequency; tests pass.

*   **Track X4: `ledger graph` Writes Transaction→Entity Edges on Commit**
    *   Status: Completed
    *   Spec: `conductor/trackX4/spec.md`
    *   Plan: `conductor/trackX4/plan.md`
    *   Goal: During `ledger commit`, write CozoDB `Affects` edges from the transaction URN to each changed file node so `ledger graph <tx-id>` returns populated results.
    *   Definition of done: `ledger graph <tx-id>` shows ≥ 1 file row for a recently committed transaction; edge writes gated on CozoDB availability; tests pass.

*   **Track X5: Security Child Node Orphan Pruning**
    *   Status: Completed
    *   Spec: `conductor/trackX5/spec.md`
    *   Plan: `conductor/trackX5/plan.md`
    *   Goal: Extend the Cedar orphan-pruning logic in `graph_loader.rs` to cascade from `policy` nodes to `principal`, `action`, and `resource` nodes derived from deleted Cedar files.
    *   Definition of done: After deleting all Cedar policies and re-indexing, `security boundaries` shows 0 entries; tests pass.

*   **Track X6: `audit <entity>` Resolves File Paths to Ledger Entities**
    *   Status: Completed
    *   Spec: `conductor/trackX6/spec.md`
    *   Plan: `conductor/trackX6/plan.md`
    *   Goal: When `<entity>` looks like a file path, `ledger audit` queries `project_file_changes` for matching transactions in addition to entity-name lookup, so `audit src/commands/hotspots.rs` works.
    *   Definition of done: File-path audit returns all transactions touching that file; entity-name lookup unchanged; tests pass.

*   **Track X7: `doctor` Shows `(not configured)` for Blank Model Name**
    *   Status: Completed
    *   Spec: `conductor/trackX7/spec.md`
    *   Plan: `conductor/trackX7/plan.md`
    *   Goal: When `embedding_model` or `generation_model` is empty string (unconfigured), `doctor` prints `(not configured)` in yellow instead of a blank-prefixed status line.
    *   Definition of done: `doctor` displays yellow `(not configured)` for blank model names; tests pass.

*   **Track X8: `hotspots trend` Human-Readable Timestamps**
    *   Status: Completed
    *   Spec: `conductor/trackX8/spec.md`
    *   Plan: `conductor/trackX8/plan.md`
    *   Goal: Format trend timestamps as `YYYY-MM-DD HH:MM UTC` in human output; JSON mode retains full RFC3339.
    *   Definition of done: `hotspots trend` shows short timestamps; `--json` unchanged; tests pass.

*   **Track X9: Empty-State Hints for `observability coverage`, `deploy impact`, and `tests`**
    *   Status: Completed
    *   Spec: `conductor/trackX9/spec.md`
    *   Plan: `conductor/trackX9/plan.md`
    *   Goal: Add empty-state hints with run-command suggestions to `observability coverage`, `deploy impact --changed`, and `changeguard tests <file>` when they return no data.
    *   Definition of done: Each command shows yellow hint + cyan run-command when empty; JSON unaffected; tests pass.

*   **Track X10: `ledger gc` No-Args UX and `ledger adr list`**
    *   Status: Completed
    *   Spec: `conductor/trackX10/spec.md`
    *   Plan: `conductor/trackX10/plan.md`
    *   Goal: `ledger gc` with no args shows styled usage (exit 0) instead of raw clap error; `ledger adr list` shows all ADRs in a table.
    *   Definition of done: `ledger gc` prints usage gracefully; `ledger adr list` works; tests pass.

*   **Track X11: `verify` Uses `cargo nextest run` When Available**
    *   Status: Completed
    *   Spec: `conductor/trackX11/spec.md`
    *   Plan: `conductor/trackX11/plan.md`
    *   Goal: `changeguard verify` probes for `cargo nextest` and uses it when available, falling back to `cargo test --workspace`. Adds `verify.prefer_nextest` config option.
    *   Definition of done: `verify` on a nextest-equipped machine runs nextest; plan output shows selected runner; tests pass.

*   **Track X12: `hotspots explain` Filters Directory-Level Coupling Noise**
    *   Status: Completed
    *   Spec: `conductor/trackX12/spec.md`
    *   Plan: `conductor/trackX12/plan.md`
    *   Goal: Filter out directory-level entries (no file extension) from Top Couplings in `hotspots explain`; show `(N directory-level entries hidden)` count note.
    *   Definition of done: Only file-path couplings appear; hidden count shown when > 0; tests pass.

*   **Track X13: `security boundaries` Summary Counts and Empty Hint**
    *   Status: Completed
    *   Spec: `conductor/trackX13/spec.md`
    *   Plan: `conductor/trackX13/plan.md`
    *   Goal: Add `[N policies | N principals | N actions | N resources]` summary to `security boundaries` header; empty state shows add-Cedar hint.
    *   Definition of done: Summary counts in header; empty hint shown when no data; JSON includes `meta.counts`; tests pass.

*   **Track X14: `scan --impact` Clean-Tree Message and Risk Reconciliation**
    *   Status: Completed
    *   Spec: `conductor/trackX14/spec.md`
    *   Plan: `conductor/trackX14/plan.md`
    *   Goal: On a clean working tree, `scan --impact` shows "Working tree is clean" instead of empty output; `Overall Risk` matches highest line-item or shows escalation note.
    *   Definition of done: Clean-tree message shown when no changes; risk reconciliation note printed; `--json` includes `tree_clean`; tests pass.

*   **Track X15: `watch` Startup Banner and Clean Exit Handling**
    *   Status: Completed
    *   Spec: `conductor/trackX15/spec.md`
    *   Plan: `conductor/trackX15/plan.md`
    *   Goal: Print repo path + Ctrl+C hint immediately on `watch` start; exit with code 0 and "Watch stopped." on Ctrl+C; ignore `.changeguard/state/` events.
    *   Definition of done: Startup banner appears; Ctrl+C exits 0; state dir changes ignored; tests pass.


## Milestone Z: Command Audit Remediation & Ollama Cloud Hardening (Completed)

*   **Track Z1: Command Audit Remediation and Ollama Cloud Hardening**
    *   Status: Completed
    *   Spec: `conductor/trackZ1/spec.md`
    *   Plan: `conductor/trackZ1/plan.md`
    *   Goal: Close all command-audit "Doesn't Work / Risks" and friction items found on 2026-06-09, with special focus on secret-safe config output, working Ollama Cloud fallback for `ask`, bounded verification health checks, structured-output consistency, and clearer UX for noisy or empty command surfaces.
    *   Definition of done: `config view --json` never emits secret values; `ask --backend local` succeeds with valid Ollama Cloud config and reports clear actionable errors for invalid credentials; `verify --health` is bounded and informative; dry-run, JSON, bridge, empty-state, and federation UX issues have tests; official Ollama API behavior is captured in regression coverage; full verification plus reinstall passes.

*   **Track Z2: `data-models impact --changed` Clean-Tree Message**
    *   Status: Completed
    *   Spec: `conductor/trackZ2/spec.md`
    *   Plan: `conductor/trackZ2/plan.md`
    *   Goal: Differentiate between clean working tree and no data models indexed in data-models impact.
    *   Definition of done: Graceful no changes message when data models exist but are unchanged; original warning when count is 0; tests pass.

*   **Track Z3: Config Diff Env Var References**
    *   Status: Completed
    *   Spec: `conductor/trackZ3/spec.md`
    *   Plan: `conductor/trackZ3/plan.md`
    *   Goal: Scan project files for environment variable references during indexing to eliminate false-negative unused declarations.
    *   Definition of done: Scan executes during incremental/full indexing, references persisted, config diff works accurately; tests pass.

*   **Track Z4: Cargo.lock Dependency Ingestion**
    *   Status: Completed
    *   Spec: `conductor/trackZ4/spec.md`
    *   Plan: `conductor/trackZ4/plan.md`
    *   Goal: Parse Cargo.lock during index --analyze-graph and populate packages as node/edges in CozoDB.
    *   Definition of done: Cargo.lock packages ingested, direct/transitive edges populated, dependencies list works; tests pass.

*   **Track Z5: Test Mapping Graph Loader**
    *   Status: Completed
    *   Spec: `conductor/trackZ5/spec.md`
    *   Plan: `conductor/trackZ5/plan.md`
    *   Goal: Port SQLite test mappings to CozoDB as nodes and validates edges during index --analyze-graph.
    *   Definition of done: Mappings loaded into CozoDB, tests <entity> command works; tests pass.

*   **Track Z6: Ledger Graph Transaction Edges**
    *   Status: Completed
    *   Spec: `conductor/trackZ6/spec.md`
    *   Plan: `conductor/trackZ6/plan.md`
    *   Goal: Write transaction -> file affects edges in CozoDB upon ledger commit.
    *   Definition of done: Affects edges populated transactional, ledger graph shows neighborhood; tests pass.

### Milestone Z Remediation (Codex Review Follow-Up)

*   **Track Z-R1: Cargo.lock Disambiguation & Schema Hardening**
    *   Status: Completed
    *   Spec: `conductor/trackZ-R1/spec.md`
    *   Plan: `conductor/trackZ-R1/plan.md`
    *   Goal: Close the test-coverage gap for Cargo.lock multi-version disambiguation and harden the parser against schema drift.
    *   Definition of done: Duplicate-version integration test passes and fails if heuristic is broken; typed deserialization path exists alongside Value fallback; zero behavior change on standard lockfiles.

*   **Track Z-R2: Ledger Adopt Path Deduplication & Defense-in-Depth**
    *   Status: Completed
    *   Spec: `conductor/trackZ-R2/spec.md`
    *   Plan: `conductor/trackZ-R2/plan.md`
    *   Goal: Eliminate redundant KG writes in `execute_ledger_adopt`, centralize synthetic-entity filtering, and harden `get_transaction_files`.
    *   Definition of done: Adopt produces exactly one node + edges per file; synthetic filter present in `write_ledger_graph_edges`; `get_transaction_files` skips non-path entities; backward-compatible API preserved.

*   **Track Z-R3: Env Schema Completeness & Regex Consolidation**
    *   Status: Completed
    *   Spec: `conductor/trackZ-R3/spec.md`
    *   Plan: `conductor/trackZ-R3/plan.md`
    *   Goal: Wire dead regexes, expand real-world env-var coverage, deduplicate patterns into `src/index/env_patterns.rs`, and make reference replacement atomic.
    *   Definition of done: All dead regexes are active; new patterns (option_env!, import.meta.env, etc.) are covered; shared module exists; atomic transaction wraps cleanup + inserts.

*   **Track Z-R4: CozoDB Parameterized Queries & Test Precision**
    *   Status: Completed
    *   Spec: `conductor/trackZ-R4/spec.md`
    *   Plan: `conductor/trackZ-R4/plan.md`
    *   Goal: Eliminate format!-based Datalog queries, add safe `CozoStorage` helpers, strengthen assertions to exact-match, and cover `--json` output.
    *   Definition of done: Zero format! interpolation in CozoDB test queries; parameterized helpers used everywhere; Z5/Z6 tests assert exact URNs; Z2 has JSON coverage.


## Milestone GF: God-File Decomposition and Boundary Hardening (Completed)

Execution guidance (added 2026-06-09 review): run these tracks **serially**, one branch at a time — every track moves large files, so parallel tracks guarantee merge churn. Hard ordering: GF3 → GF6 → GF7. GF1, GF2, GF4, GF5, GF8 are independent of each other, but GF1 and GF8 both touch `DeadCodeFinding`/`ConfidenceFactor` in `src/impact/packet.rs`, so whichever runs second must rebase on the first. Every track is a `REFACTOR`-category ledger transaction: `ledger start` in Phase 0, `ledger commit` at finalization.

*   **Track GF1: Impact Packet Domain Type Split**
    *   Status: Completed
    *   Spec: `conductor/trackGF1/spec.md`
    *   Plan: `conductor/trackGF1/plan.md`
    *   Goal: Decompose `src/impact/packet.rs` into focused domain modules for core packet metadata, changed files, risk, verification, coverage, observability, contracts, services, deployments, dependencies, security, and serialization helpers without changing the public `ImpactPacket` schema.
    *   Definition of done: Existing imports continue to compile through compatibility re-exports; all packet JSON snapshots and integration behavior remain stable; compile-time fan-in risk is reduced by module boundaries; full verification plus reinstall passes.

*   **Track GF2: Config Model Domain Split**
    *   Status: Completed
    *   Dependencies: GF1 optional
    *   Spec: `conductor/trackGF2/spec.md`
    *   Plan: `conductor/trackGF2/plan.md`
    *   Goal: Split `src/config/model.rs` into domain-specific config modules and isolate environment/dotenv/default resolution from pure config data types.
    *   Definition of done: Config serialization, aliases, secret redaction, env precedence, and validation behavior are unchanged; domain modules have focused tests; full verification plus reinstall passes.

*   **Track GF3: Native Graph Loader Phase Extraction**
    *   Status: Completed
    *   Dependencies: GF1 optional
    *   Spec: `conductor/trackGF3/spec.md`
    *   Plan: `conductor/trackGF3/plan.md`
    *   Goal: Break the 1300-line `build_native_graph` procedure in `src/index/graph_loader.rs` into explicit graph loading phases with testable helpers and unchanged CozoDB output.
    *   Coverage note: `GraphLoadContext` struct + nine `phase_*` functions extracted. Phase functions are private and invoked through `build_native_graph`; integration tests cover the full end-to-end pipeline. Per-phase unit tests were deferred — see `docs/GF-review.md` for the rationale.

*   **Track GF4: Ledger Database Query Domain Split**
    *   Status: Completed
    *   Dependencies: GF2 optional
    *   Spec: `conductor/trackGF4/spec.md`
    *   Plan: `conductor/trackGF4/plan.md`
    *   Goal: Decompose `src/ledger/db.rs` by query domain while preserving `LedgerDb` as the stable facade for transaction lifecycle, drift, search, ADR, enforcement, federation, provenance, and graph-link operations.
    *   Definition of done: Existing `LedgerDb` call sites are not forced to migrate in the same track; query-domain tests use isolated temp repositories/databases; ledger hooks and drift lifecycle continue to work; full verification plus reinstall passes.

*   **Track GF5: CLI Command Definition and Dispatch Split**
    *   Status: Completed
    *   Dependencies: GF2, GF4 optional
    *   Spec: `conductor/trackGF5/spec.md`
    *   Plan: `conductor/trackGF5/plan.md`
    *   Goal: Split `src/cli.rs` into command-group argument modules and dispatch helpers while preserving the `run_with` entry point, clap contract, aliases, JSON behavior, and command help text.
    *   Definition of done: `changeguard --help` and command-level help remain stable except intentional grouping; dispatch smoke tests cover all command groups; full verification plus reinstall passes.

*   **Track GF6: Index Orchestrator Capability Split**
    *   Status: Completed
    *   Dependencies: GF3
    *   Spec: `conductor/trackGF6/spec.md`
    *   Plan: `conductor/trackGF6/plan.md`
    *   Goal: Split `src/index/orchestrator.rs` into focused indexing capabilities for file discovery, tree-sitter parsing, index lifecycle, extraction delegation, centrality, services, KG build delegation, and SQL row helpers. (Note: the facade struct is `ProjectIndexer`; SCIP/semantic/docs-export orchestration lives in `src/commands/index.rs` and belongs to GF7.)
    *   Definition of done: `ProjectIndexer` remains the stable public facade; each capability has focused tests or smoke coverage; graph/search freshness behavior remains unchanged; full verification plus reinstall passes.

*   **Track GF7: Index Command Mode Extraction**
    *   Status: Completed
    *   Dependencies: GF3, GF6
    *   Spec: `conductor/trackGF7/spec.md`
    *   Plan: `conductor/trackGF7/plan.md`
    *   Goal: Extract `src/commands/index.rs` mode handlers for docs, contracts, analyze-graph, semantic, semantic-dry-run, incremental, check, SCIP, export-docs, and the `--fast` Gemini extraction path into navigable functions or modules with shared option normalization (including `--concurrency` and `--strict`).
    *   Definition of done: All `changeguard index` modes preserve CLI behavior, progress output, JSON/script safety, and state side effects; integration tests cover every mode path; full verification plus reinstall passes.

*   **Track GF8: Dead-Code Analysis Provider Boundary Tightening**
    *   Status: Completed
    *   Dependencies: GF3, GF6 optional
    *   Spec: `conductor/trackGF8/spec.md`
    *   Plan: `conductor/trackGF8/plan.md`
    *   Goal: Continue the RE2 provider-pattern direction in `src/impact/analysis/dead_code.rs` by splitting evidence collection, confidence scoring, filtering, and report rendering into focused modules.
    *   Definition of done: Dead-code confidence scores remain deterministic; existing tests remain green and gain focused provider coverage; no new false-positive deletion recommendations are introduced; full verification plus reinstall passes.

*   **Track GF9: Python AST Parser Extraction**
    *   Status: Completed
    *   Dependencies: none (GF9 establishes the pattern GF10 follows)
    *   Spec: `conductor/trackGF9/spec.md`
    *   Plan: `conductor/trackGF9/plan.md`
    *   Goal: Split `src/index/languages/python.rs` (1,471 lines, ~1,007 production) into focused extraction modules by concern — `symbols.rs`, `routes.rs`, `calls.rs`, `models.rs`, `observability.rs`, `common.rs` — in a `python/` directory with `python.rs` retained as the facade (GF8 `dead_code.rs` pattern). Mirrors the RE5 Rust decomposition module names.
    *   Definition of done: `python.rs` is a ≤30-line facade; all extraction concerns in dedicated modules; all public import paths unchanged; full verification plus reinstall passes.

*   **Track GF10: TypeScript AST Parser Extraction**
    *   Status: Completed
    *   Dependencies: GF9 (pattern established there first)
    *   Spec: `conductor/trackGF10/spec.md`
    *   Plan: `conductor/trackGF10/plan.md`
    *   Goal: Split `src/index/languages/typescript.rs` (1,362 lines, ~945 production) into the same extraction-concern module shape as GF9 — `symbols.rs`, `routes.rs`, `calls.rs`, `models.rs`, `observability.rs`, `common.rs` — in a `typescript/` directory with `typescript.rs` retained as the facade.
    *   Definition of done: `typescript.rs` is a ≤30-line facade; all extraction concerns in dedicated modules; all public import paths unchanged; full verification plus reinstall passes.

*   **Track GF11: CI Gates Platform Split**
    *   Status: Completed
    *   Dependencies: none
    *   Spec: `conductor/trackGF11/spec.md`
    *   Plan: `conductor/trackGF11/plan.md`
    *   Goal: Split `src/index/ci_gates.rs` (1,045 lines, ~984 production) into per-platform parser modules — `github_actions.rs`, `gitlab_ci.rs`, `circleci.rs`, `makefile.rs` — in a `ci_gates/` directory with `ci_gates.rs` retained as the facade holding shared infrastructure (`CIGateExtractor`, `ParsedCIGate`, `CIGateStats`, path helpers) and dispatch. Per-platform characterization golden tests are written BEFORE any move (GF1 precedent).
    *   Definition of done: `ci_gates.rs` contains no platform-specific parsing; each CI platform has its own module with parser logic and a golden test; all public symbols unchanged; full verification plus reinstall passes.

*   **Track GF12: Local Model Client Split**
    *   Status: Completed
    *   Dependencies: none
    *   Spec: `conductor/trackGF12/spec.md`
    *   Plan: `conductor/trackGF12/plan.md`
    *   Goal: Split `src/local_model/client.rs` (1,170 lines, ~583 production) by endpoint provider into a `client/` child directory (GF4 `db.rs` pattern): `types.rs`, `ollama.rs`, `openai.rs`, `gemini.rs`, `cloud.rs`, `util.rs`. `client.rs` remains the facade carrying the verified public API: `complete`, `gemini_complete`, `ping_completions`, `has_ollama_cloud_fallback`, `ChatMessage`, `CompletionOptions`.
    *   Definition of done: `client.rs` holds only public API, dispatch, and re-exports; each endpoint protocol in its own child module; all six public symbols importable at existing paths; full verification plus reinstall passes.

*   **Track GF13: Entrypoint Language Detector Split**
    *   Status: Completed
    *   Dependencies: GF9, GF10 (structural symmetry — implement those first)
    *   Spec: `conductor/trackGF13/spec.md`
    *   Plan: `conductor/trackGF13/plan.md`
    *   Goal: Split `src/index/entrypoint.rs` (1,045 lines, ~798 production) into per-language detector modules — `rust.rs`, `typescript.rs`, `python.rs` — in an `entrypoint/` directory with `entrypoint.rs` retained as the facade holding shared types (`EntrypointKind`, `EntrypointStats`, `SymbolClassification`) and re-exports. Completes language-layer symmetry with RE5 and GF9/GF10.
    *   Definition of done: Each language detector in its own module with co-located tests; `entrypoint.rs` holds shared types and re-exports only; all existing import paths unchanged; full verification plus reinstall passes.

*   **Track GF14: Ledger Command Group Split**
    *   Status: Completed
    *   Dependencies: none
    *   Spec: `conductor/trackGF14/spec.md`
    *   Plan: `conductor/trackGF14/plan.md`
    *   Goal: Split `src/commands/ledger.rs` (1,006 lines, zero tests) into command-group modules — `lifecycle.rs` (start/commit/rollback/atomic/resume), `maintenance.rs` (gc/hook_repair/reconcile/adopt), `registration.rs` (register_rule/register_validator), `reporting.rs` (status/export_provenance) — in a `ledger/` directory with `ledger.rs` retained as the facade. Pure helpers gain unit tests; handlers rely on the existing integration suite (they are cwd-dependent and not unit-testable).
    *   Definition of done: `ledger.rs` is a pure facade; 13 handlers split across 4 groups; pure-helper unit tests added; all import paths unchanged; full verification plus reinstall passes.


## Milestone KD: Knowledge Graph Deepening & CozoDB-redux Upgrade (Completed)

*   **Track KD1: CozoDB-Redux Dependency Upgrade**
    *   Status: Completed
    *   Spec: `conductor/trackKD1/spec.md`
    *   Plan: `conductor/trackKD1/plan.md`
    *   Goal: Upgrade the `cozo` and `cozo-sys` dependencies in `Cargo.toml` to the latest GitHub-pushed `CozoDB-redux` repository version. Verify environmental health using `changeguard doctor` and ensure full test suite compatibility under the new engine.
    *   Definition of done: Cargo compilation succeeds; `changeguard doctor` reports graph active; full verification test suite passes.

*   **Track KD2: Transitive Closure Reachability in KGProvider**
    *   Status: Completed
    *   Spec: `conductor/trackKD2/spec.md`
    *   Plan: `conductor/trackKD2/plan.md`
    *   Goal: Refactor `src/impact/enrichment/kg_provider.rs` to replace the hardcoded 1-hop and 2-hop reachability checks with a parameterized recursive Datalog transitive closure query up to a configurable maximum depth.
    *   Definition of done: `kg_provider.rs` uses a single recursive query; configurable depth limit is enforced; reachability tests remain green.

*   **Track KD3: Declarative Logical & Security Checks in Datalog**
    *   Status: Completed
    *   Spec: `conductor/trackKD3/spec.md`
    *   Plan: `conductor/trackKD3/plan.md`
    *   Goal: Move imperative Rust-side verification checks—specifically security boundary authorization checks in `security.rs` and entrypoint-to-sink/unreachable check logic—into declarative Datalog rules in CozoDB.
    *   Definition of done: Imperative loops are replaced by expressive Datalog queries; correctness and boundary violations are validated via database constraints/rules; existing security tests pass.

*   **Track KD4: PageRank-Based Churn & Centrality Risk Scoring**
    *   Status: Completed
    *   Spec: `conductor/trackKD4/spec.md`
    *   Plan: `conductor/trackKD4/plan.md`
    *   Goal: Run CozoDB's native PageRank algorithm over the code dependency/call graph. Blend the resulting symbol centrality metrics with raw file change frequencies to rank codebase risk.
    *   Definition of done: PageRank-based centrality is calculated on-the-fly or cached in CozoDB; overall node risk scores scale with graph centrality; tests verify score determinism.


## Milestone E: Engineering Coverage (In Progress)

*   **Track CG-F1: Upgrade Gemini to 3.1 GA & FTS5 Sanitization**
    *   Status: Completed
    *   Spec: `conductor/trackCG-F1/spec.md`
    *   Plan: `conductor/trackCG-F1/plan.md`
    *   Goal: Upgrade the project to Gemini 3.1 GA models and implement robust FTS5 query sanitization.
    *   Key additions: `gemini-3.1` model standardization, `src/util/query.rs` (sanitizer), integration into `bridge query`.

*   **Track E0-1: Ledger Verification Gate Enforcement**
    *   Status: Completed
    *   Spec: `conductor/trackE0-1/spec.md`
    *   Plan: `conductor/trackE0-1/plan.md`
    *   Goal: Implement and wire the Ledger Verification Gate enforcement logic to block high-risk commits lacking verification.
    *   Key additions: `TransactionManager` gate logic, `--force` bypass, `verify_to_commit` config, `tests/ledger_enforcement.rs`.

*   **Track E0-2: Hotspot Complexity Fallback**
    *   Status: Completed
    *   Spec: `conductor/trackE0-2/spec.md`
    *   Plan: `conductor/trackE0-2/plan.md`
    *   Goal: Implement a two-tier complexity lookup for hotspots, falling back to project-wide symbols when impact-run data is missing.
    *   Key additions: `query_file_complexities` helper, graceful degradation for missing `project_symbols` table, batched SQLite lookups.

*   **Track E0-3: Federated Dependency Matching**
    *   Status: In Progress
    *   Spec: `conductor/trackE0-3/spec.md`
    *   Plan: `conductor/trackE0-3/plan.md`
    *   Goal: Implement `SymbolMatcher` caching and edge-case tests in `src/federated/scanner.rs` to optimize cross-repo symbol matching.

*   **Track CG-F3: Fix ledger note**
    *   Status: In Progress
    *   Spec: `conductor/trackCG-F3/spec.md`
    *   Plan: `conductor/trackCG-F3/plan.md`
    *   Goal: Restore the `ledger note` command in `src/cli/args.rs` and remove any deprecation warnings.

## Workflow


1.  **Plan**: `@architecture-planner` creates `conductor/trackN/plan.md`.
2.  **Push Plan**: Commit and push plan to `main`.
3.  **Implement**: `@generalist` (Implementer) creates a new branch and works on the task.
4.  **Review**: `@rust-triage-specialist` or `@frontend-reviewer` (Reviewer) audits the branch.
5.  **Iteration**: If review fails, Implementer fixes.
6.  **Merge**: If review passes, create PR or merge into `main`.
7.  **Next**: Start next track.
ext track.

# Changeguard Phase 2 Implementation Plan (Hardened)

## 0. Executive Summary of Phase 2

Phase 1 established ChangeGuard as a deterministic guardrail running on strict structural parsing and targeted verification. Phase 2 extends ChangeGuard into a **data-driven intelligence platform**. It introduces Temporal Awareness (change coupling), Complexity Scoring, Probabilistic Verification (predictive failure), and Federated Impact Analysis (cross-repository mapping).

In strict adherence to `docs/Engineering.md`, this plan dictates exactly how to implement these advanced features without violating the Core Implementation Principles. We must preserve single-binary execution, local-first state, strict `Result` propagation via `miette`, and reject premature abstraction.

### 0.1 Summary of Hardening Changes Over Prior Draft

1. **Corrects stale dependency pins.** `linfa` updated from stale 0.7.1 to 0.8.1. `tower-lsp-server` v0.23.0 API migration to `ls-types` documented. `arborist-metrics` 0.1.2 retained as conditional option with maturity risk assessment (brand-new crate, first published March 2026, zero community adoption).
2. **Adds an explicit Phase 1 anti-goal migration path.** Phase 1 banned tokio/async and background daemons. Phase 2 introduces both. This plan contains a formal Architectural Migration section explaining the boundary and constraints.
3. **Adds Rust Implementation Rules and Determinism Contract** as required by `docs/Engineering.md`.
4. **Adds cold-start, lifecycle, and graceful-degradation requirements** missing from the prior draft.
5. **Replaces silent fallback behaviors with explicit partial-result reporting** per `docs/Engineering.md`.
6. **Adds configurable commit depth limits** to prevent unbounded history crawling.
7. **Adds daemon lifecycle management** (PID files, graceful shutdown, stale lock detection).
8. **Tightens federated intelligence safety** (symlink escape, schema validation, cycle caps).
9. **Adds Windows-specific daemon edge cases** (named pipes, service lifecycle).
10. **Corrects "Gemeni" typo to "Gemini" throughout.**
11. **Documents rusqlite 0.39.0 breaking changes** affecting Phase 2 DB usage (statement validation, disabled `u64`/`usize` `ToSql`/`FromSql`).
12. **Adds MSRV requirement, `cargo deny` update guidance, and CI workflow additions for Phase 2.**

---

## 1. Product Intent for Phase 2

ChangeGuard Phase 2 is **not** an autonomous AI agent. It remains an orchestration layer. Its new responsibilities are:

- Exposing the *history* of the code alongside the *structure* of the code.
- Visualizing risk density ("Hotspots") across large repositories.
- Short-circuiting verification loops by identifying the *most likely* tests to fail first.
- Feeding contextual intelligence directly into the developer's IDE via an LSP-compliant daemon.

---

## 2. Core Implementation Principles (Addendum)

In addition to the Phase 1 non-negotiable principles, Phase 2 mandates:

1. **Idiomatic Error Visibility**: All new subsystems MUST propagate errors using `miette::Result`. `unwrap()` and `expect()` are globally banned in production logic.
2. **Deterministic Modeling**: Statistical correlation (e.g., in Probabilistic Verification) must be deterministic given the same SQLite state. Random seeds MUST be hardcoded in tests and configurable in production.
3. **No Unbounded Background Work**: The LSP daemon must be strictly resource-limited and idle-aware to avoid starving the primary workspace.
4. **Data Isolation**: Federated cross-repo dependency reading is read-only. We do not support multi-repo atomic commits.
5. **KISS in Machine Learning**: Use standard statistical correlation (`linfa` or native Rust algorithms) before jumping to neural networks. Prefer native implementations when `linfa` adds disproportionate dependency weight.

### 2.1 Rust Implementation Rules

These rules apply to all Phase 2 code:

- Public fallible functions MUST return `Result<T, E>`.
- User-facing command handlers MUST return `miette::Result<()>`.
- Internal orchestration MAY use `anyhow::Result<T>` where typed errors are not worth the complexity.
- `unwrap()`, `expect()`, and unchecked assumptions are FORBIDDEN in production code. Permitted only in tests or impossible-by-construction cases with documented `// SAFETY:` comments.
- Prefer `Result` propagation with `?`.
- Use `match` for branch clarity when multiple failure modes need user-visible handling.
- Use `Option` only when absence is expected and non-exceptional.
- Convert lower-level errors into actionable command-level diagnostics with context.
- Panics in normal runtime paths are a defect.

### 2.2 Determinism Contract

All Phase 2 subsystems MUST obey these determinism requirements:

- **Stable Ordering**: All emitted file lists, symbol lists, reasons, commands, and report sections MUST be sorted deterministically (alphabetical by path, then by line number when paths collide).
- **Stable Packet Schema**: Impact packet extensions introduced in Phase 2 MUST carry a `schema_version` field. Test fixtures MUST pin schema versions.
- **No Silent Fallback Heuristics**: If a parser, model, or scanner fails, the tool MUST record partial results explicitly with a `degraded: true` annotation and a `degradation_reason` string. It MUST NOT quietly invent replacement behavior.
- **Deterministic Default Plans**: Given the same repo state, config, and SQLite contents, all outputs (verification plans, hotspot rankings, probability orderings) MUST be identical.
- **Clock Sensitivity**: Timestamps MUST NOT be embedded into comparison-sensitive test fixtures unless explicitly normalized. Use `util/clock.rs` for injectable time sources.

### 2.3 Actionable Error Standard

All command errors MUST include:

- **What failed**: the operation, file, or subsystem.
- **Where it failed**: path, command, dependency, or symbol involved.
- **Likely cause**: when reasonably determinable.
- **Next step for the user**: a concrete remediation action.

Example quality bar:

- Bad: `"Failed to compute hotspots"`
- Good: `"Hotspot analysis failed: git history contains only 3 commits (minimum 10 required for meaningful coupling data). Run 'changeguard hotspots' again after accumulating more history, or use '--min-commits 3' to force analysis with reduced confidence."`

---

## 3. Architectural Migration: Phase 1 Anti-Goals Revisited

Phase 1 Section 2.2 explicitly banned "Tokio/async-first architecture" and "background daemon outside the active CLI process." Phase 2 introduces both for the LSP daemon. This section explains why and constrains the scope.

### 3.1 Why the Anti-Goals Are Relaxed

The LSP protocol is inherently asynchronous and connection-oriented. `tower-lsp-server` requires a Tokio runtime. There is no viable synchronous LSP implementation in the Rust ecosystem. The daemon is the *only* Phase 2 subsystem that requires async.

### 3.2 Containment Rules

1. **Tokio is confined to `src/daemon/`**. No other Phase 2 module may depend on Tokio or any async runtime. The daemon binary entry point is isolated from the main CLI binary entry point.
2. **The daemon is opt-in**. It is started only by explicit `changeguard daemon` invocation. No Phase 2 feature requires the daemon to function. CLI commands (hotspots, scan, impact, verify) remain fully synchronous.
3. **The daemon MUST NOT write to the DB**. It reads from SQLite in WAL mode with read-only connections only. The CLI process retains exclusive write authority.
4. **Consider a separate binary**. If Tokio's compile-time cost becomes problematic, the daemon MAY be extracted to a `changeguard-daemon` binary in the same workspace. The CLI binary MUST NOT link Tokio.
5. **Feature gate Tokio**. The `daemon` feature flag MUST gate all Tokio dependencies so that `cargo build --no-default-features` produces a daemon-free binary.

---

## 4. Architecture Boundaries for Phase 2

The Phase 2 architecture expands the system into the following strict boundaries:

1. `impact/temporal/`: Parses git history, computes logical coupling. MUST NOT depend on AST parsing. MUST NOT depend on Tokio.
2. `index/metrics/`: Computes cognitive and cyclomatic complexity scoring via tree-sitter. See Phase 18 for implementation path decision (`arborist-metrics` vs native).
3. `verify/probability/`: Statistics engine mapping SQLite test histories to specific symbols. MUST NOT depend on Tokio.
4. `daemon/`: The `tower-lsp-server` engine mapping intelligence queries to editor capabilities. MUST run on a feature-gated Tokio runtime. MUST use read-only SQLite.
5. `impact/federated/`: Reads schemas from sibling `.changeguard/schema.json` instances. MUST NOT follow symlinks. MUST NOT depend on Tokio.

### 4.1 SRP Constraints (per Engineering.md)

- `impact/temporal.rs`: Computes co-change affinity scores from commit vectors ONLY. Does not query AST or index data.
- `index/metrics.rs`: Computes complexity integers from tree-sitter ASTs ONLY. Does not access git history.
- `verify/probability.rs`: Reads SQLite test history and outputs probability-ordered verification plans ONLY. Does not execute tests.
- `daemon/server.rs`: Translates LSP protocol messages to ChangeGuard queries ONLY. Does not compute scores itself.
- `impact/federated.rs`: Reads and validates sibling schema files ONLY. Does not write to siblings.

---

## 5. Hardened Phase 2 Repository Layout

```text
src/
├── commands/
│   ├── hotspots.rs          # Visualizes complex vs temporal density
│   ├── export_schema.rs     # Exports schema.json for federation
│   └── daemon.rs            # Spawns the LSP server (feature-gated)
├── daemon/                  # Feature-gated: requires "daemon" feature
│   ├── mod.rs
│   ├── server.rs            # tower-lsp-server LanguageServer trait impl
│   ├── state.rs             # Read-only SQLite access + stale-data detection
│   ├── handlers.rs          # Hover and CodeLens processing
│   └── lifecycle.rs         # PID file management, graceful shutdown
├── index/
│   └── metrics.rs           # Cognitive/Cyclomatic complexity via tree-sitter
├── impact/
│   ├── temporal.rs          # Logical coupling detection map
│   ├── federated.rs         # Cross-repo schema discovery
│   └── export.rs            # Schema generation for federation
└── verify/
    └── probability.rs       # Test correlation logic

tests/
├── temporal_coupling.rs
├── complexity_scoring.rs
├── hotspot_ranking.rs
├── probability_ordering.rs
├── daemon_lifecycle.rs      # Feature-gated
├── federated_discovery.rs
└── fixtures/
    ├── coupling/            # Hardcoded commit history fixtures
    ├── complexity/          # Known-complexity source files
    ├── federation/          # Mock sibling repo layouts
    └── probability/         # Mock test history SQLite DBs
```

---

## 6. Pinned Dependency Baseline (2026 Standard)

The Phase 2 dependency stack is governed by the same `cargo deny` and `cargo audit` discipline as Phase 1.

### 6.1 Toolchain

- **Rust**: 1.95.0+ (same MSRV as Phase 1)
- **Edition**: 2024

### 6.2 Verified Dependencies

```toml
[dependencies]
# Phase 17: History Parsing (same as Phase 1, shared dependency)
gix = "0.81.0"

# Phase 18: Complexity Scoring (conditional — see Phase 18 and Section 6.4)
# Option A (preferred if arborist-metrics spike passes):
arborist-metrics = "0.1.2"
# Option B (fallback — native implementation, no additional dependency):
# Complexity computed inline using Phase 1 tree-sitter queries.

# Phase 20: Statistics (optional, for probabilistic verification)
linfa = "0.8.1"
linfa-logistic = "0.8.1"

# Phase 21: IDE Integration (feature-gated behind "daemon")
[dependencies.tower-lsp-server]
version = "0.23.0"
optional = true

[dependencies.tokio]
version = "1.x"
features = ["rt-multi-thread", "io-std", "macros"]
optional = true

[features]
default = []
daemon = ["dep:tower-lsp-server", "dep:tokio"]
```

### 6.3 Dependency Audit Notes

| Dependency | Pinned | Verified | Notes |
|:---|:---|:---|:---|
| `gix` | 0.81.0 | 2026-04 | Latest on docs.rs. Rapid release cycle; pin in `Cargo.lock`. |
| `arborist-metrics` | 0.1.2 | 2026-04 | **HIGH RISK — see Section 6.4.** First published Mar 31, 2026. 3 releases in 3 days. 0 stars, 0 forks, 0 dependent packages, 1 contributor. Functional fit is excellent (cognitive + cyclomatic + SLOC across 12 languages via tree-sitter). Maturity is not. |
| `linfa` | 0.8.1 | 2026-04 | Breaking change from 0.7.x: ndarray upgraded to 0.16. Do NOT use 0.7.x. |
| `linfa-logistic` | 0.8.1 | 2026-04 | Required sub-crate for logistic regression. Must match `linfa` version. |
| `tower-lsp-server` | 0.23.0 | 2026-04 | Community fork of `tower-lsp`. **Breaking**: v0.23.0 switched from `lsp-types` to `ls-types` (community fork). All imports must use `tower_lsp_server::ls_types::*`, NOT `lsp_types::*`. |
| `tokio` | 1.x | 2026-04 | Feature-gated. Only linked when `daemon` feature is enabled. |

### 6.4 `arborist-metrics` Risk Assessment

`arborist-metrics` 0.1.2 is a real crate that provides exactly what Phase 18 needs: multi-language cognitive complexity (SonarSource), cyclomatic complexity (McCabe), and SLOC via tree-sitter across 12 languages. However, it carries significant adoption risk:

- **Age**: First published March 31, 2026 (< 1 month old at time of writing).
- **Community**: 0 stars, 0 forks, 0 watchers, 0 dependent packages, 1 contributor.
- **Release velocity**: 3 releases in 3 days (0.1.0, 0.1.1, 0.1.2). This suggests active early development but no stability plateau.
- **License**: MIT/Apache-2.0 (compatible).
- **tree-sitter compatibility**: Must be verified against Phase 1's tree-sitter 0.26.8 before adoption. If `arborist-metrics` pins an incompatible tree-sitter version, it cannot be used.

**Decision framework:**

1. Before starting Phase 18, run `cargo add arborist-metrics@0.1.2` in a throwaway branch and verify it compiles against the Phase 1 tree-sitter 0.26.8 dependency. If it causes version conflicts, reject it immediately.
2. If it compiles, write a spike test computing complexity on the Phase 1 fixture files. Evaluate correctness against hand-calculated complexity scores.
3. If the spike passes, adopt `arborist-metrics` for Phase 18. This avoids reinventing a well-scoped wheel.
4. If the spike fails, OR if the crate is abandoned/yanked before Phase 18 begins, implement complexity scoring natively using tree-sitter queries as the fallback path.
5. Regardless of the decision, the `index/metrics.rs` module MUST wrap complexity computation behind a ChangeGuard-owned trait (`trait ComplexityScorer`) so the implementation can be swapped without changing callers.

### 6.5 Phase 2 Dependency Cautions

1. **`tower-lsp-server` v0.23.0 API migration**: The `LanguageServer` trait no longer requires `#[async_trait]`. The `symbol()` method now returns `WorkspaceSymbolResponse`. Use `ls_types` (community fork), not `lsp-types` (unmaintained). See the [v0.23.0 changelog](https://github.com/tower-lsp-community/tower-lsp-server/releases/tag/v0.23.0).
2. **`linfa` 0.8.x breaking changes**: The 0.8.0 release upgraded to `ndarray` 0.16 and added `linfa-ensemble`. If Phase 1 uses `ndarray` elsewhere, versions must be aligned.
3. **`gix` velocity**: `gix` releases frequently under 0.x semver. Pin via `Cargo.lock` and treat upgrades as coordinated changes with integration testing.
4. **`rust-code-analysis` was evaluated and rejected**: Mozilla's tree-sitter-based complexity crate (v0.0.25) has not been updated since January 2023 and depends on obsolete tree-sitter versions incompatible with Phase 1's tree-sitter 0.26.x. Do not adopt it.

### 6.6 rusqlite 0.39.0 Breaking Changes (Phase 2 Impact)

Per `docs/breaking.md`, rusqlite 0.39.0 (already adopted in Phase 1) introduced breaking changes that affect Phase 2's new DB queries:

- **Statement validation**: `Connection::execute` now rejects SQL strings containing more than one statement. Phase 2 MUST NOT use multi-statement `execute()` calls for any new schema (temporal coupling tables, probability history tables, federation cache). Use separate `execute()` calls or `prepare()` + step.
- **Disabled unsigned integer defaults**: `u64` and `usize` `ToSql`/`FromSql` impls are disabled by default. Phase 2 MUST use `i64` for all SQLite integer columns, or explicitly enable the `u64` feature on `rusqlite` if unsigned is required.
- **Multiple-statement `prepare()`**: `prepare()` now checks for multiple statements. All new Phase 2 migrations MUST use single-statement strings.

### 6.7 `cargo deny` Configuration Updates

When Phase 2 dependencies are added, update `deny.toml`:

- Add `arborist-metrics` (if adopted) to the `[licenses]` allow list (MIT/Apache-2.0).
- Add `linfa`, `linfa-logistic`, and their transitive `ndarray` dependency tree to the allow list.
- Add `tower-lsp-server` and `tokio` to the allow list (feature-gated).
- Verify no new `unsafe` crates are introduced transitively by running `cargo deny check advisories`.

---

## 7. Threat Model and Safety Posture (Phase 2 Addendum)

### 7.1 LSP Daemon

- **Daemon Hijacking**: The daemon MUST bind strictly to `stdio` IPC streams (stdin/stdout). TCP binding is PROHIBITED in v1. If TCP is added later, it MUST bind to `127.0.0.1` only with a per-session random token.
- **Resource Exhaustion**: The daemon MUST enforce a maximum concurrent request limit (default: 4). Requests exceeding the limit receive a `ServerNotInitialized` error, not a queue.
- **Stale Daemon**: On startup, the daemon writes a PID file to `.changeguard/daemon.pid`. On `changeguard daemon`, if a PID file exists and the process is alive, refuse to start with an actionable error. If the PID file exists but the process is dead, delete the stale PID file and proceed.

### 7.2 Federated Intelligence

- **Cross-Repo Poisoning**: Malformed or maliciously crafted `.changeguard/schema.json` files in sibling directories could panic the parser. Strict JSON schema validation is mandatory. Parse with `serde_json` inside a `catch_unwind` boundary as a defense-in-depth measure.
- **Symlink Escape**: The scanner MUST NOT follow symlinks when traversing `../`. Use `std::fs::symlink_metadata` (not `std::fs::metadata`) and skip entries where `file_type().is_symlink()` is true.
- **Path Traversal**: After resolving `../`, the scanner MUST canonicalize the result and verify it is exactly one directory level above the repo root. Reject paths that escape further.
- **Privacy (Federated)**: Secret redaction MUST occur *before* a federated schema is generated and stored locally. The export command MUST strip all values from detected env/config patterns, retaining only keys.

### 7.3 Probabilistic Verification

- **Model Poisoning**: An adversary who controls test history in SQLite could skew probability rankings. The model MUST cap the influence of any single symbol at a configurable maximum weight (default: 0.5).
- **Cold-Start Safety**: When the database contains fewer than 10 verification runs, probabilistic ordering MUST be disabled entirely with an explicit diagnostic: `"Insufficient history for probabilistic ordering (have: N, need: 10). Using sequential verification."` The tool MUST NOT attempt correlation with insufficient data.

---

## 8. Edge Cases to Design For (Global Phase 2)

Implementers must ensure the following are handled deterministically:

### 8.1 Git History (Phase 17)

- **Shallow clones**: `gix` may encounter objects it cannot parse at shallow clone boundaries. Return `Err` with `"Repository is a shallow clone; temporal analysis requires full history. Run 'git fetch --unshallow' to enable coupling analysis."`.
- **Rebases and forced pushes**: The temporal engine MUST gracefully handle detached commits or rewritten histories. If a commit referenced in the coupling cache no longer exists, invalidate the cache and recompute.
- **Unbounded history**: The commit crawl depth MUST be configurable (default: 1,000, max: 10,000). Repos with >100k commits must not cause unbounded memory growth.
- **Brand-new repos**: Repos with zero or very few commits (< 10) MUST produce a graceful diagnostic, not an empty or misleading coupling map.
- **Merge commits**: Merge commits and commits touching >50 files MUST be excluded from the affinity dataset by default. The threshold MUST be configurable.
- **Linear vs branched history**: The engine MUST handle both linear and heavily branched histories. First-parent-only traversal SHOULD be the default, with a `--all-parents` flag for opt-in full traversal.

### 8.2 Complexity Scoring (Phase 18)

- **AST corruption during active editing**: If the complexity parser encounters syntax errors during a user's active keystrokes (daemon mode), it MUST return a provisional score accompanied by a `complexity_warning: "AST parse incomplete"` annotation. It MUST NOT crash.
- **Unsupported languages**: Files in languages without tree-sitter grammars MUST receive `complexity: null` with `reason: "language not supported"`, not a default score of zero.
- **Generated code**: Files matching configurable generated-code patterns (default: `*.generated.*`, `*.pb.*`, `*_generated.rs`) SHOULD be excluded from complexity scoring by default.
- **`arborist-metrics` version drift**: If adopted, pin the exact version in `Cargo.lock`. If the crate's tree-sitter dependency drifts from Phase 1's tree-sitter 0.26.8, treat this as a blocking issue and switch to the native fallback.

### 8.3 Daemon Lifecycle (Phase 21)

- **SQLite contention**: The LSP daemon and the CLI process will contend for `.changeguard/db.sqlite3`. The daemon MUST use read-only SQLite connections in WAL mode. If the database is locked, the daemon MUST retry with exponential backoff (max 3 attempts, 100ms/200ms/400ms) then return stale cached data with a `data_stale: true` annotation.
- **Malformed client URIs**: VS Code may send URIs like `file:///c%3A/` vs `C:\`. The daemon MUST normalize URIs using `tower-lsp-server`'s `UriExt` before path resolution. Mismatches MUST produce a diagnostic log entry, not a crash.
- **Windows named pipes**: On Windows, stdio-based IPC through VS Code works. Named pipe support is NOT required for v1 but SHOULD be noted as a future consideration.
- **Graceful shutdown**: The daemon MUST handle `shutdown` and `exit` LSP notifications. On `exit`, clean up the PID file and release all SQLite connections within 1 second.

### 8.4 Federated Intelligence (Phase 22)

- **Identical sibling names/namespaces**: If two sibling repos export schemas with the same package name, the federation scanner MUST disambiguate by directory name and report the ambiguity as a warning.
- **Circular dependencies**: A strict cycle-detection mechanism MUST cap federation depth at 1 (direct siblings only). No transitive federation in v1.
- **Missing or corrupt schema.json**: A sibling directory containing an invalid schema.json MUST be skipped with a warning, not abort the entire federation scan.

### 8.5 DB Migrations (All Phases)

- **Phase 2 schema additions**: New tables (temporal coupling, probability history, federation cache) MUST be added via `rusqlite_migration` (same as Phase 1). Each migration MUST use single-statement SQL strings per rusqlite 0.39.0 requirements.
- **Backward compatibility**: Phase 2 migrations MUST NOT alter Phase 1 tables. Additive only.
- **Interrupted migration**: If a Phase 2 migration is interrupted, the DB MUST remain usable for Phase 1 operations. Phase 2 features degrade with a diagnostic: `"Phase 2 schema not initialized. Run 'changeguard init --upgrade' to apply."`.

---

## 9. High-Level Delivery Sequence

1. Phase 17: Temporal Intelligence (History Extraction).
2. Phase 18: Complexity Indexing.
3. Phase 19: Hotspot Visualization.
4. Phase 20: Probabilistic Verification.
5. Phase 21: LSP-Lite ChangeGuard Daemon.
6. Phase 22: Federated Intelligence.
7. Phase 23: Advanced Narrative Reporting (Gemini).

---

## Phase 17: Temporal Intelligence (Co-Change Mapping)

### Objective

Identify "Logical Coupling" between files that frequently change together in git history, even when they lack structural imports.

### Deliverables

- `src/impact/temporal.rs`

### Functional Requirements

- Use `gix` (0.81.0) to crawl commit history from HEAD.
- Default crawl depth: 1,000 commits. Configurable via `config.toml` key `temporal.max_commits` (range: 10–10,000).
- Calculate an affinity score: if file B appears in >75% of commits involving file A (configurable threshold via `temporal.coupling_threshold`), they are considered Coupled.
- Obey SRP: the `gix` commit parser MUST only return `Vec<CommitFileSet>` structs. Scoring is a separate function.
- First-parent traversal by default. `--all-parents` flag for opt-in full traversal.

### Edge Cases

- `gix` encountering unparseable objects at shallow clone boundaries: return actionable error with remediation guidance.
- Linear history vs heavily branched/merge-commit history: exclude merge commits by default.
- Commits involving >50 files (auto-generated code, dependency updates): exclude from affinity dataset. Threshold configurable via `temporal.max_files_per_commit`.
- Repos with <10 commits: return `TemporalResult::InsufficientHistory { commits_found: N, commits_needed: 10 }`.
- Repos with zero commits (unborn branch): return `TemporalResult::NoHistory`.
- Force-pushed histories with orphaned commits: skip commits that cannot be resolved and log a warning.

### Acceptance Criteria

- `unwrap()` is forbidden. Uses `miette::Result`.
- Deterministic: same commit set produces identical affinity map.
- All emitted file pairs are sorted alphabetically for stable output.
- Partial failures (e.g., one unparseable commit among 1,000) are annotated, not fatal.

### Verification Gate

- Unit tests passing a hardcoded list of `CommitFileSet` vectors to verify deterministic affinity calculation.
- Fixture test with a synthetic git repo containing known coupling patterns.
- Edge case tests: shallow clone, empty repo, giant commit exclusion.

---

## Phase 18: Complexity Indexing

### Objective

Measure cognitive and cyclomatic complexity for functions and structs to weight impact risks.

### Deliverables

- `src/index/metrics.rs`

### Implementation Decision: `arborist-metrics` vs Native

Phase 18 has two viable implementation paths. The decision MUST be made at implementation time based on the spike test described in Section 6.4.

**Option A — `arborist-metrics` 0.1.2 (preferred if spike passes):**

- Provides SonarSource cognitive complexity, McCabe cyclomatic complexity, and SLOC across 12 languages via tree-sitter.
- Reduces implementation effort to integration and wrapping.
- Risk: brand-new crate (< 1 month old, 0 community adoption). May be abandoned, yanked, or break on tree-sitter version updates.

**Option B — Native implementation (fallback):**

- Implement complexity scoring using tree-sitter queries from the Phase 1 indexer.
- Count branching nodes (if/else, match arms, for, while, loop, &&, ||) for cyclomatic complexity.
- Count branching nodes with nesting-depth weighting for cognitive complexity (per SonarSource's specification).
- Zero additional dependencies, fully under our control.
- Higher implementation cost but lower ongoing risk.

**Regardless of path chosen:**

- Output an integer complexity score alongside symbol metadata in the existing symbol storage model.
- Wrap complexity computation behind a ChangeGuard-owned trait:

```rust
pub trait ComplexityScorer {
    fn score_file(&self, path: &Utf8Path, source: &[u8], language: Language)
        -> miette::Result<FileComplexity>;
}
```

This trait allows swapping the implementation (from `arborist-metrics` to native or vice versa) without changing callers.

- Supported languages: Rust, TypeScript, Python (matching Phase 1 parser coverage).

### Edge Cases

- AST nodes that do not map cleanly to complexity heuristics: assign `Complexity::Unknown` with reason string.
- Language not supported by complexity indexer (e.g., config files, markdown): return `Complexity::NotApplicable`.
- Syntax errors in source files: compute partial complexity from the parseable portion and annotate with `ast_incomplete: true`.
- Generated/minified files: skip by default (configurable pattern list).
- Very large files (>10,000 lines): apply a complexity cap and annotate `complexity_capped: true`.

### Acceptance Criteria

- Missing or failed metrics degrade gracefully to `Complexity::Unknown` rather than stopping execution.
- Deterministic: same source file always produces the same score.
- Implementation is swappable via the `ComplexityScorer` trait.

### Verification Gate

- `cargo test` analyzing known complex vs simple fixture functions across Rust, TypeScript, and Python.
- Golden-file tests comparing complexity output against hand-calculated scores.
- Edge case tests: syntax errors, unsupported languages, generated files.

---

## Phase 19: Hotspot Identification (Risk Density)

### Objective

Combine change frequency (Phase 17) with structural complexity (Phase 18) to output Risk Maps.

### Deliverables

- `src/commands/hotspots.rs`

### Functional Requirements

- Implement `changeguard hotspots` to render a table of the top N most "explosive" files (default: 10, configurable via `--top N`).
- Time window configurable via `--since` (default: 30 days, format: `30d`, `12w`, `6m`).
- Hotspot score = `coupling_frequency * complexity_score`. Both factors are normalized to [0.0, 1.0] before multiplication. Normalization: `value / max(all_values)`. When `max` is 0, the factor is treated as 0.0 and annotated.
- Adhere to YAGNI: use CLI tables (`output/table.rs`). No HTML generation.
- Support `--format json` for machine-readable output.

### Edge Cases

- Repo is completely new (no history): display `"No temporal data available. Run 'changeguard hotspots' after accumulating commit history."`.
- All complexities are zero or unknown: display files ranked by coupling frequency only, annotated with `"Complexity data unavailable; ranked by change frequency only."`.
- History too shallow (< 10 commits): display explicit diagnostic per Section 2.3.
- Time window contains zero commits: display `"No commits found in the specified time window."`.
- All files have identical scores: ranking is deterministic via alphabetical file path tiebreaker.

### Acceptance Criteria

- Output gracefully explains conditions where density cannot be calculated.
- Ranking is deterministic: ties are broken by alphabetical file path.
- JSON output matches a versioned schema.

### Verification Gate

- Formatting test for the CLI output table.
- Golden-file test for JSON output schema.
- Edge case tests: empty history, all-zero complexity, tie-breaking.

---

## Phase 20: Probabilistic Verification

### Objective

Order verification commands dynamically using historical failure probability rather than blind rule sets.

### Deliverables

- `src/verify/probability.rs`

### Functional Requirements

- Correlate symbols changed to failed verification executions stored in `.changeguard/db.sqlite3`.
- Implement logistic regression using `linfa-logistic` (0.8.1) or a native Rust implementation if `linfa`'s dependency weight is disproportionate.
- Update `verify/plan.rs` to accept an optional probability-ordered plan that reorders commands.
- The model MUST produce deterministic results given the same SQLite rows. Use a hardcoded random seed (42) for any stochastic initialization.
- All new DB queries MUST use `i64` for integer columns (not `u64`/`usize`) per rusqlite 0.39.0 defaults.

### Cold-Start Strategy

When the database contains fewer than 10 completed verification runs:

1. Probabilistic ordering is DISABLED entirely.
2. The tool emits an explicit diagnostic: `"Probabilistic verification ordering requires at least 10 historical runs (found: N). Using sequential ordering."`.
3. Sequential (rule-based) ordering from Phase 11 is used as the fallback.
4. This is NOT a silent fallback. The diagnostic appears in both CLI output and the verification report JSON.

### Edge Cases

- Database has been purged (no training data): cold-start behavior applies.
- `linfa` encounters singular matrix or mathematically unresolvable correlations: the model MUST return an explicit `ProbabilityResult::ModelFailure { reason }` and fall back to sequential ordering WITH a diagnostic.
- All historical runs passed (no failures to learn from): the model MUST return `ProbabilityResult::InsufficientVariance` and use sequential ordering.
- Symbols renamed between model training and current analysis: stale symbol mappings are ignored (no negative or phantom scores).

### Acceptance Criteria

- The model MUST produce deterministic results given the same input SQLite rows.
- Model failures are reported explicitly in the verification report, NOT silently captured in logs. (The prior draft's "silently captured in logs" violates Engineering.md's "No Silent Fallback" principle.)
- Sequential ordering fallback always includes a diagnostic explaining WHY probabilistic ordering was not used.

### Verification Gate

- Mock data yielding 100% predictable test ordering.
- Cold-start test: empty database produces sequential ordering with diagnostic.
- Model failure test: singular matrix input produces explicit fallback.

---

## Phase 21: LSP-Lite ChangeGuard Daemon

### Objective

Provide real-time intelligence overlay in the developer's IDE using `tower-lsp-server` (0.23.0).

### Deliverables

- `src/commands/daemon.rs`
- `src/daemon/server.rs`
- `src/daemon/handlers.rs`
- `src/daemon/lifecycle.rs`
- `src/daemon/state.rs`

### Prerequisites

- All daemon code is gated behind the `daemon` Cargo feature.
- The daemon binary entry point is isolated from the main CLI.

### Functional Requirements

- Start via `changeguard daemon`. Communicate over stdio (stdin/stdout).
- Respond to `textDocument/codeLens` with risk scores for functions in the current file.
- Respond to `textDocument/hover` with impact summaries for hovered symbols.
- MUST run on a constrained Tokio runtime (`tokio::runtime::Builder::new_multi_thread().worker_threads(2)`).
- MUST use read-only WAL SQLite connections.
- MUST write a PID file to `.changeguard/daemon.pid` on startup.
- MUST clean up PID file on shutdown/exit.

### API Migration Note

`tower-lsp-server` v0.23.0 introduced a breaking change: it now uses `tower_lsp_server::ls_types` instead of the unmaintained `lsp_types` crate. All LSP type imports MUST use:

```rust
use tower_lsp_server::ls_types::*;
```

The `LanguageServer` trait no longer requires `#[async_trait]`. Use native async trait methods directly.

### Edge Cases

- Stale PID file from crashed daemon: detect dead process, remove stale PID, proceed with startup.
- The LSP client sends malformed URIs (e.g., `file:///c%3A/` vs `C:\`): use `UriExt::to_file_path()` for normalization. Log mismatches but do not crash.
- The daemon encounters `SQLITE_BUSY` because `changeguard scan` is actively writing: retry with exponential backoff (100ms, 200ms, 400ms), then return cached data with `data_stale: true`.
- VS Code restarts without sending `shutdown`: the daemon MUST detect broken stdin and self-terminate within 5 seconds.
- Windows: stdio-based IPC works through VS Code's extension host. Named pipe support is deferred to a future release.

### Acceptance Criteria

- Daemon never crashes on client URI mismatches.
- Read-only WAL SQLite access prevents DB deadlocks.
- PID file lifecycle is correct (created on start, removed on stop, stale detection works).
- Feature-gated: `cargo build` without `--features daemon` does not link Tokio.

### Verification Gate

- Spawning the daemon and passing standard LSP initialization payloads in tests.
- PID file lifecycle tests (create, stale detection, cleanup).
- SQLite contention simulation tests.

---

## Phase 22: Federated Intelligence

### Objective

Track cross-component impact across local sibling directories.

### Deliverables

- `src/impact/federated.rs`
- `src/impact/export.rs`
- `src/commands/export_schema.rs`

### Functional Requirements

- Add a `changeguard export-schema` command that generates `schema.json` identifying public interfaces (exported functions, public types, module boundaries). This command MUST be registered in `src/cli.rs` alongside existing subcommands.
- Implement a scanner that traverses exactly one level up (`../`) to detect sibling directories containing `.changeguard/schema.json`.
- The scanner MUST NOT follow symlinks. Use `std::fs::symlink_metadata` for all directory entries.
- The scanner MUST canonicalize resolved paths and reject any that are not exactly one directory level above the repo root.
- Schema files are parsed with `serde_json`. Parsing failures for any single sibling MUST be logged as a warning and skipped, not abort the scan.

### Edge Cases

- Sibling repos with identical package names: disambiguate by directory name, report ambiguity as warning.
- Circular dependencies (A references B references A): capped at depth 1 by design. No transitive federation.
- Secrets leaked into `schema.json`: the export command MUST strip all detected secret values (env var values, API keys, connection strings) before writing. Only keys/names are retained.
- Symlinks in `../` pointing outside the parent directory: skip with warning.
- Large number of siblings (>20): cap scan at 20 siblings with a configurable limit.
- Malformed schema.json (valid JSON but wrong structure): validate against a versioned JSON schema. Reject non-conforming files with a diagnostic.

### Acceptance Criteria

- A strict cycle-detection mechanism caps federation depth at 1.
- Redaction rules execute on all schema data exports.
- No symlink following.
- Schema validation rejects malformed files gracefully.

### Verification Gate

- Fixture integration testing across two mock sibling repositories inside tempdir.
- Symlink escape test.
- Malformed schema test.
- Secret redaction test.

---

## Phase 23: Advanced Narrative Reporting (Gemini)

### Objective

Implement proactive markdown synthesis and improved secret management via Gemini CLI integration.

### Deliverables

- `src/gemini/narrative.rs`

### Functional Requirements

- Generate a human-readable "Executive Summary" markdown document representing the impact packet, including hotspot highlights and coupling insights from Phase 2 data.
- The summary is generated by passing the impact packet to Gemini CLI in `analyze` mode with a structured narrative prompt.
- Augment secret redaction with pattern signatures mapped from `gitleaks` patterns (embedded as a static pattern list, NOT a runtime dependency on `gitleaks`).
- Implement a token budget estimator: if the impact packet exceeds 80% of the configured Gemini context window (default: 128k tokens), truncate low-priority sections (verification stdout, unchanged file metadata) before submission.

### Edge Cases

- Extremely long impact packets exceeding Gemini's sequence length: the token budget estimator triggers truncation with a `"Packet truncated for Gemini submission"` annotation in the summary.
- Over-aggressive redaction mangling critical JSON/TOML configs: redaction operates on VALUE positions only, never on keys or structural syntax.
- Gemini CLI absent: the command degrades gracefully with `"Gemini CLI not found. Install Gemini CLI to enable narrative summaries."`.
- Gemini CLI exits non-zero: capture stderr, report the error, and save the raw impact packet as the fallback deliverable.

### Acceptance Criteria

- Summary generation is deterministic given the same impact packet (Gemini output variability is acceptable; the INPUT to Gemini must be deterministic).
- Preserves YAML/JSON/TOML syntax boundaries during redaction.
- Token budget estimation prevents submission failures.

### Verification Gate

- Verify prompt rendering is deterministic via golden-file tests.
- Verify redaction preserves config syntax via fixture tests.
- Verify token budget truncation via packet-size simulation tests.

---

## 10. Milestones

### Milestone E — Historical Intelligence

Complete:

- Phase 17 (Temporal Intelligence)
- Phase 18 (Complexity Indexing)
- Phase 19 (Hotspot Visualization)

### Milestone F — Predictive Verification

Complete:

- Phase 20 (Probabilistic Verification)

### Milestone G — IDE Integration

Complete:

- Phase 21 (LSP-Lite Daemon)

### Milestone H — Cross-Repo and Reporting

Complete:

- Phase 22 (Federated Intelligence)
- Phase 23 (Advanced Narrative Reporting)

---

## 11. Testing Strategy (Phase 2 Addendum)

### Unit Tests

Use for:

- Affinity score computation
- Complexity score computation (both `arborist-metrics` and native paths)
- `ComplexityScorer` trait contract tests
- Probability model determinism
- Schema validation
- Token budget estimation
- Secret redaction patterns

### Fixture Tests

Use for:

- Coupling detection from synthetic commit histories
- Complexity scoring from known-complexity source files
- Hotspot ranking from combined fixture data
- Federation schema discovery from mock sibling layouts
- Daemon URI normalization

### Integration Tests

Use for:

- `hotspots` command end-to-end
- `export-schema` command end-to-end
- Daemon startup/shutdown lifecycle (feature-gated)
- Probabilistic verification ordering with mock DB
- DB migration forward-compatibility (Phase 2 migrations on Phase 1 DB)

### Manual Validation

Always validate on:

- Windows 11 + PowerShell (especially daemon stdio behavior)
- WSL2 Ubuntu
- Ubuntu native if available

### CI Workflow Additions

Add the following to `.github/workflows/` for Phase 2:

- `cargo test --features daemon` (run daemon-specific tests)
- `cargo build --no-default-features` (verify daemon-free binary compiles)
- `cargo deny check` (verify new dependencies pass license and advisory checks)
- Fixture-based complexity golden-file tests in the standard `cargo test` suite

---

## 12. AI Implementation Protocol (Phase 2 Addendum)

In addition to the Phase 1 AI implementation discipline:

1. The `daemon` feature MUST be implemented last (Phase 21). Do not introduce Tokio or async code until Phases 17–20 are stable.
2. Before starting Phase 18, run the `arborist-metrics` spike test described in Section 6.4. Document the result. If the spike fails, proceed with the native implementation. If the spike passes, adopt `arborist-metrics` behind the `ComplexityScorer` trait.
3. Probabilistic verification (Phase 20) SHOULD start with a native implementation (simple logistic regression or Bayesian estimator) before pulling in `linfa`. If the native version is adequate, `linfa` may be deferred.
4. Do not introduce `ndarray` unless `linfa` is adopted. If `linfa` is adopted, `ndarray` version must be 0.16.x to match `linfa` 0.8.x.
5. All new SQL queries MUST be reviewed against rusqlite 0.39.0 breaking changes (single-statement `execute()`, `i64` integer types) before merge.
6. Register all new subcommands (`hotspots`, `daemon`, `export-schema`) in `src/cli.rs` during the phase that introduces them, not deferred.

---

## 13. Final Implementation Warning

Phase 2 introduces genuinely complex subsystems (statistical modeling, LSP protocol, cross-repo scanning). The risk of over-engineering is higher than in Phase 1.

The most important success criteria remain:

- predictable behavior
- inspectable outputs
- safe local state
- clear platform handling
- bounded resource consumption
- explainable intelligence

The daemon is the riskiest subsystem. It should be the last thing built and the first thing cut if schedule pressure demands it. Every Phase 2 feature MUST work without the daemon.

Reliability comes first. Sophistication comes second. Intelligence comes third.

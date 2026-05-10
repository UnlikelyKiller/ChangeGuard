# Technical Specification: Track 51-1 — Probabilistic Reachability & Dead Code Detection

## 1. Objective
Implement confidence-based dead code detection by blending three local signals:
- **Knowledge Graph reachability** (CozoDB reverse reachability from entrypoints)
- **Git activity recency** (file-level commit frequency via `gix`)
- **Test coverage history** (symbol-to-test mappings and test outcome embeddings)

The result is a per-symbol confidence score in `[0.0, 1.0]` indicating the likelihood that a symbol is dead code, surfaced in the impact pipeline and available via a standalone CLI command.

## 2. Architecture & Data Model

### 2.1 DeadCodeConfig (`src/config/model.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeadCodeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_dead_code_confidence_threshold")]
    pub confidence_threshold: f64,
    #[serde(default = "default_git_inactivity_days")]
    pub git_inactivity_days: u32,
    #[serde(default = "default_reachability_weight")]
    pub reachability_weight: f64,
    #[serde(default = "default_git_activity_weight")]
    pub git_activity_weight: f64,
    #[serde(default = "default_test_coverage_weight")]
    pub test_coverage_weight: f64,
}
```

All weights are normalized internally so users may specify any positive values.

### 2.2 ConfidenceScorer (`src/impact/analysis/dead_code.rs`)

```rust
pub struct ConfidenceScorer<'a> {
    cozo: Option<&'a CozoStorage>,
    storage: &'a StorageManager,
    config: &'a DeadCodeConfig,
    repo_path: &'a Path,
}

impl<'a> ConfidenceScorer<'a> {
    pub fn new(
        cozo: Option<&'a CozoStorage>,
        storage: &'a StorageManager,
        config: &'a DeadCodeConfig,
        repo_path: &'a Path,
    ) -> Self;

    /// Score a single symbol. Returns `None` if the symbol is an entrypoint itself.
    pub fn score_symbol(&self, symbol: &Symbol, file_path: &Path) -> Result<Option<DeadCodeFinding>>;

    /// Score all symbols in a file.
    pub fn score_file(&self, file_path: &Path) -> Result<Vec<DeadCodeFinding>>;

    /// Full-repo scan (used by the standalone `dead-code` command).
    pub fn scan_repo(&self, limit: usize) -> Result<Vec<DeadCodeFinding>>;
}
```

### 2.3 Scoring Algorithm

**Signal A — Graph Reachability (CozoDB)**
1. Identify entrypoint node IDs from SQLite (`project_symbols` where `entrypoint_kind` in `ENTRYPOINT`, `HANDLER`, `PUBLIC_API`).
2. Run a CozoDB fixed-point reachability query backward from entrypoints through `edge` relations.
3. Symbols with **zero reverse paths** from any entrypoint receive `reachability_score = 1.0`. All others receive `0.0`.

**Signal B — Git Activity (`gix`)**
1. For the file containing the symbol, traverse `HEAD` history up to `temporal.max_commits`.
2. Count commits whose tree diff touches the file.
3. `git_activity_score = 1.0` if 0 commits in the last `git_inactivity_days`, linearly decaying to `0.0` at the threshold.

**Signal C — Test Coverage (SQLite)**
1. Query `test_mapping` for rows where `tested_symbol_id` matches the symbol ID.
2. If `test_mapping` is empty, fall back to `test_outcome_history` joined via `embeddings` to see if the symbol's file ever appeared in a historical diff with test coverage.
3. `test_coverage_score = 1.0` if no mappings/outcomes exist, else `0.0`.

**Blend Formula**
```rust
let sum = config.reachability_weight
        + config.git_activity_weight
        + config.test_coverage_weight;

let confidence = (reachability_weight * reachability_score
                + git_activity_weight * git_activity_score
                + test_coverage_weight * test_coverage_score)
                / sum;
```

A finding is emitted only when `confidence >= confidence_threshold` (default `0.75`).

### 2.4 ImpactPacket Extension (`src/impact/packet.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ConfidenceFactor {
    UnreachableFromEntrypoints,
    GitInactive { days_since_last_commit: u32 },
    NoTestCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeadCodeFinding {
    pub symbol_name: String,
    pub file_path: PathBuf,
    pub confidence: f64,
    pub factors: Vec<ConfidenceFactor>,
    pub recommendation: String,
}
```

Add to `ImpactPacket`:
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub dead_code_findings: Vec<DeadCodeFinding>,
```

### 2.5 Risk Provider (`src/impact/providers/dead_code_provider.rs`)

Implements `RiskProvider`. If a changed symbol appears in `dead_code_findings` with `confidence >= threshold`, emits an advisory reason:
> "Advisory: changed symbol `<name>` in `<file>` is likely dead code (confidence: X%)"

Weight: `0` (advisory only; does not affect risk level calculation).

### 2.6 Enrichment Hook (`src/impact/enrichment/dead_code.rs`)

Implements `EnrichmentProvider`. Runs after `analyze_risk()` (audit5 pattern). Gated by `config.dead_code.enabled`. Populates `packet.dead_code_findings` for all changed files in the packet.

### 2.7 CLI Integration

- **`Impact` command** (`src/cli.rs`, `src/commands/impact.rs`): Add `--dead-code` flag. When present, the impact pipeline runs the scorer against changed files and appends findings.
- **`DeadCode` command** (`src/cli.rs`, `src/commands/dead_code.rs`): New top-level command:
  ```
  changeguard dead-code [--threshold <f64>] [--limit <usize>]
  ```
  Performs a full-repo proactive scan and prints a sorted table.

## 3. API Contracts

### Confidence Scoring Algorithm
- **Input**: `Symbol`, file path, live CozoDB handle (optional), SQLite handle, `DeadCodeConfig`.
- **Output**: `Option<DeadCodeFinding>` — `None` for entrypoints or when `confidence < threshold`.
- **Idempotency**: Same repo state + same config → identical scores (deterministic sorting and stable git traversal).
- **Graceful Degradation**: If CozoDB is unavailable, `reachability_score` falls back to structural-edge analysis from SQLite (`structural_edges` table). If `gix` history is shallow, cap `git_activity_score` at the available history.

### CLI Integration
- `--dead-code` on `impact` is a no-op when `dead_code.enabled = false` in config.
- The standalone `dead-code` command ignores the config `enabled` flag (user explicitly requested it) but still respects thresholds and weights.
- Exit code `0` even when no findings (command succeeded); exit code `1` only on internal errors.

### ImpactPacket Extension
- `dead_code_findings` is sorted in `finalize()` by: `confidence` descending → `file_path` ascending → `symbol_name` ascending.
- `dead_code_findings` is cleared in `truncate_for_context()` Phase 3.
- Empty vector is absent from JSON output.

## 4. Testing Strategy

| Test | Type | Assertion |
|---|---|---|
| `DeadCodeConfig::default()` | Unit | `enabled == false`, thresholds and weights match defaults |
| Config backward compat | Unit | M7-era config deserializes with `dead_code` disabled |
| `score_symbol` unreachable | Unit | `reachability_score == 1.0` |
| `score_symbol` reachable | Unit | `reachability_score == 0.0` |
| `score_symbol` inactive file | Unit | `git_activity_score > 0.0` |
| `score_symbol` active file | Unit | `git_activity_score == 0.0` |
| `score_symbol` no test mapping | Unit | `test_coverage_score == 1.0` |
| `score_symbol` with test mapping | Unit | `test_coverage_score == 0.0` |
| Blend with default weights | Unit | `confidence` within `1e-6` of expected |
| `dead_code_findings` sorted in finalize | Unit | confidence descending, then path, then name |
| `dead_code_findings` cleared in truncate | Unit | empty after Phase 3 |
| Serialization roundtrip populated | Unit | JSON → deserialize → serialize matches |
| Serialization roundtrip empty | Unit | absent from JSON |
| `--dead-code` flag populates packet | Integration | findings non-empty when flag present |
| `dead_code` command full scan | Integration | returns findings across repo, respects `--limit` |
| Provider emits advisory | Integration | risk reasons contain "likely dead code" |
| Provider weight is zero | Integration | risk level unchanged by dead code advisory |
| CozoDB unavailable fallback | Integration | falls back to SQLite `structural_edges` without panic |

## 5. Dependencies & Risks

| Dependency | Risk | Mitigation |
|---|---|---|
| CozoDB reverse reachability query | Large graphs may be slow | Limit impact-path scan to changed files only; full-repo scan obeys `--limit` |
| `gix` file-history traversal | Large repos with deep history | Reuse `temporal.max_commits` cap; cache commit counts per file within a single run |
| Empty `test_mapping` / `test_outcome_history` | False positives on coverage | Reduce `test_coverage_weight` to `0` dynamically when both tables are empty |
| Graph not yet built | CozoDB empty or missing | Skip reachability scoring with a warning; rely on git + test signals alone |
| Module restructure (`analysis.rs` → `analysis/`) | Risk of breaking existing imports | Preserve `pub fn analyze_risk` signature exactly; move body into `analysis/mod.rs` |

## 6. Success Criteria

- [ ] `changeguard impact --dead-code` produces findings for changed files with confidence scores and human-readable recommendations.
- [ ] `changeguard dead-code` produces a sorted table of likely dead symbols across the entire repo.
- [ ] All new code follows the `miette::Diagnostic` + `thiserror` pattern with zero `unwrap()` / `expect()`.
- [ ] `cargo test` passes with no regressions; new module achieves ≥90% line coverage.
- [ ] Serialization roundtrip tests pass for all new packet fields.
- [ ] Deterministic output: repeated runs on identical repo state produce byte-identical `ImpactPacket` JSON.
- [ ] Config backward compatibility: existing configs without `[dead_code]` deserialize and behave identically.

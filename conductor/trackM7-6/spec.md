# Specification: Track M7-6 — ADR Staleness Detection

## Objective
Flag retrieved ADRs exceeding a configurable age threshold, with severity tiers, recently-updated exemption, and multi-source age detection.

## Components

### 1. Staleness Computation (`src/retrieval/query.rs` extend)

```rust
pub fn compute_staleness(file_path: &Path, threshold_days: u32) -> Option<u32>
```

Age detection sources (in priority order):
1. File mtime from `std::fs::metadata`
2. ADR frontmatter `date:` field (YAML frontmatter in markdown)
3. `created:` metadata line in ADR body
4. Git-based fallback: `git log --follow --format=%ct` for last modification

Use the **most recent** date found — an ADR that was edited yesterday is not stale even if created 2 years ago.

### 2. Recently-Updated Exemption

If file mtime is within 30 days: never flag as stale, regardless of creation date or threshold.

### 3. Severity Tiers

| Age | Tier | Message |
|---|---|---|
| < threshold_days | None | (no flag) |
| threshold_days – threshold_days×2 | Warning | "ADR '{title}' is {age} days old — may need review" |
| > threshold_days×2 | Critical | "ADR '{title}' is {age} days old — significantly stale, may not reflect current architecture" |

### 4. Type Extension

```rust
// Extend existing RelevantDecision
pub struct RelevantDecision {
    pub file_path: String,
    pub heading: Option<String>,
    pub excerpt: String,
    pub similarity: f32,
    pub rerank_score: Option<f32>,
    // M7-6 addition:
    pub staleness_days: Option<u32>,
    pub staleness_tier: Option<StalenessTier>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum StalenessTier { Warning, Critical }
```

### 5. Impact Enrichment

Staleness is a **warning**, not a risk reason. It does not change `risk_level`. It appears in:
- Ask context: appended to the relevant decisions block with staleness annotation
- Human output: staleness tier shown inline with each matched ADR

## Test Specifications

| Test | Assertion |
|---|---|
| ADR mtime > threshold_days | `staleness_days` populated |
| ADR mtime < threshold_days | `staleness_days` is `None` |
| ADR with frontmatter `date:` | Date from frontmatter used |
| ADR edited 5 days ago | Exempt (within 30-day window) |
| ADR > threshold_days×2 → Critical tier | `staleness_tier == Some(StalenessTier::Critical)` |
| ADR with no date metadata or mtime | Git-based fallback attempted |
| `[coverage.adr_staleness].enabled = false` | No staleness computation |
| Serialization roundtrip | `staleness_days` and `staleness_tier` survive JSON roundtrip |

## Constraints & Guidelines

- **TDD**: Tests written before implementation.
- **Warning, not risk**: Staleness does not change `risk_level`.
- **Config-driven**: `[coverage.adr_staleness].enabled = false` → no computation.
- **Determinism**: Staleness computed from mtime/frontmatter only (no network, no external process).

## Hardening Additions (in plan)

| Addition | Reason |
|---|---|
| Multi-source age detection | mtime → frontmatter → metadata → git, most recent wins |
| Recently-updated exemption | An ADR edited yesterday is not stale |
| Severity tiers (365/730 days) | Differentiate "may need review" from "significantly stale" |
| Git-based age fallback | Generated/unversioned docs may lack filesystem dates |
| Staleness in ask context | Brief explanation of why staleness matters for the current change |

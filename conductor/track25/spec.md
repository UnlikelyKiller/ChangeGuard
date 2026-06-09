# Track 25 Spec: Hotspot Identification (Risk Density)

## Objective
Combine change frequency (from git history) with structural complexity (from AST analysis) to identify "Hotspots" — files and symbols that are both complex and change frequently.

## Deliverables
- `src/commands/hotspots.rs`: Implementation of the `changeguard hotspots` command.
- `HotspotEngine`: Logic for combining temporal and complexity metrics into a unified Risk Density score.
- CLI table output for hotspot ranking.

## Functional Requirements
1. **Scoring Logic**:
   - `Risk Density = (Complexity Score * Weight) + (Change Frequency * Weight)`.
   - Complexity Score: Max(Cognitive, Cyclomatic) scaled to 0-1 range.
   - Change Frequency: Percentage of total commits in window (e.g., last 100 commits) that touch the file.
2. **Ranking**:
   - Output top 10 hotspots by default.
   - Sort descending by Risk Density.
3. **Filtering**:
   - Allow filtering by directory or language.
   - Option to show "Logical Neighbors" (highly coupled files) for each hotspot.
4. **Determinism**:
   - Rankings must be deterministic for a fixed git history and local state.
5. **Output Formats**:
   - Human-readable table (default).
   - JSON (for CI/CD or external visualization).

## Internal API
```rust
pub struct Hotspot {
    pub path: Utf8PathBuf,
    pub score: f32,
    pub complexity: f32,
    pub frequency: f32,
}

pub fn calculate_hotspots(
    history: &[CommitFileSet],
    complexity_records: &[FileComplexity]
) -> Vec<Hotspot>;
```

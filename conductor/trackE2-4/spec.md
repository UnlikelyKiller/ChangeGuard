# Specification: Track E2-4 - Critical Path Analysis

## 1. Objective
Use the structural call graph (from E2-1) and entry point data (from E1-4) to compute centrality for each symbol: how many entry points can reach it via call edges. Store centrality in a new `symbol_centrality` table (Migration M16). Integrate centrality into hotspots (optional column when data exists) and risk scoring (+15 for high-centrality symbols). This track depends on E2-1 (`structural_edges`) and E1-4 (entry points).

## 2. Deliverables

### 2.1 Centrality Computation
- **Target file**: New `src/index/centrality.rs`
- **Details**: Implement centrality computation using the call graph from `structural_edges` and entry points from `project_symbols` (where `entrypoint_kind` column is `ENTRYPOINT` or `HANDLER`):
  1. Load all `structural_edges` from the database.
  2. Build an adjacency list (forward edges: caller -> callee).
  3. For each entry point (from `project_symbols` where `entrypoint_kind IN ('ENTRYPOINT', 'HANDLER')`), perform a breadth-first search (BFS) through the call graph, recording which symbols are reachable.
  4. For each reachable symbol, increment its `entrypoints_reachable` count.
  5. Compute betweenness centrality approximately: count how many shortest paths from entry points pass through each symbol. This is optional; the primary metric is `entrypoints_reachable`.
  6. Store results in `symbol_centrality`.

### 2.2 Symbol Centrality Table (Migration M16)
- **Target file**: `src/state/migrations.rs`
- **Details**: Add to Migration M16 (shared with E2-1, E2-2, E2-3) creating the `symbol_centrality` table:
  ```sql
  CREATE TABLE IF NOT EXISTS symbol_centrality (
      id                      INTEGER PRIMARY KEY AUTOINCREMENT,
      symbol_id               INTEGER NOT NULL REFERENCES project_symbols(id),
      file_id                 INTEGER NOT NULL REFERENCES project_files(id),
      entrypoints_reachable   INTEGER NOT NULL DEFAULT 0,
      betweenness             REAL DEFAULT 0.0,
      last_computed_at        TEXT NOT NULL,
      FOREIGN KEY (symbol_id) REFERENCES project_symbols(id),
      FOREIGN KEY (file_id) REFERENCES project_files(id)
  );
  CREATE INDEX IF NOT EXISTS idx_symbol_centrality_symbol
      ON symbol_centrality(symbol_id);
  CREATE INDEX IF NOT EXISTS idx_symbol_centrality_file
      ON symbol_centrality(file_id);
  CREATE INDEX IF NOT EXISTS idx_symbol_centrality_reachable
      ON symbol_centrality(entrypoints_reachable);
  ```

### 2.3 Hotspots Integration
- **Target file**: `src/impact/hotspots.rs`
- **Details**:
  - Add an optional `centrality` column to the hotspot output (when `structural_edges` data exists and `symbol_centrality` is populated).
  - When computing hotspots, join with `symbol_centrality` to include the `entrypoints_reachable` count for each symbol's file.
  - The `--centrality` flag on `changeguard hotspots` explicitly requests centrality data. When `structural_edges` is empty, display "Centrality: N/A" instead of a number.
  - The JSON output (`--json`) includes `centrality` and `entrypoints_reachable` fields when available.

### 2.4 Impact Integration
- **Target file**: `src/impact/analysis.rs`
- **Details**:
  - When a changed symbol has `entrypoints_reachable > 5` in `symbol_centrality`, add risk weight up to 15 points within the Historical Hotspot category (max 30 points).
  - Add risk reason: `"High centrality: {symbol_name} reachable from {N} entry points"`.
  - Query `symbol_centrality` by joining on `symbol_id` to match the changed symbol's `project_symbols` row.
  - The threshold of 5 entry points is configurable via the existing risk scoring configuration.
  - If `symbol_centrality` table is empty, skip centrality-based risk scoring entirely.

### 2.5 Index Command Integration
- **Target file**: Command handler for `changeguard index`
- **Details**: Centrality computation runs via `changeguard index --analyze-graph`, not during every `index` run. The `--analyze-graph` flag triggers centrality computation after `structural_edges` (E2-1) and entry point labeling (E1-4) are complete. If either is missing, skip centrality computation and log an info message. A standard `changeguard index` run does NOT compute centrality.

## 3. Constraints & Guidelines
- **Depends on E2-1 and E1-4**: Centrality requires `structural_edges` (call graph) and `project_symbols` with `entrypoint_kind` labels. If either is empty, centrality computation is skipped gracefully.
- **Centrality is opt-in via `--analyze-graph`**: Centrality is NOT computed during a standard `changeguard index` run. Users must explicitly run `changeguard index --analyze-graph` to populate `symbol_centrality`.
- **No performance regression**: Centrality computation must complete in under 5 seconds for a repo with 50,000 edges. BFS traversal uses an adjacency list, not recursive graph traversal.
- **BFS depth cap**: Limit BFS traversal to 20 hops from each entry point. Symbols deeper than 20 hops from any entry point are not counted as reachable.
- **Cycle safety**: Use a visited set in BFS to prevent infinite loops on cyclic call graphs.
- **Graceful degradation**: If `structural_edges` is empty, `symbol_centrality` remains empty. Hotspots show "Centrality: N/A". Impact scoring skips centrality risk.
- **Backward-compatible schema**: The `symbol_centrality` table is additive. No existing table is modified.

## 4. Edge Cases

| Edge Case | Handling |
|-----------|----------|
| No call graph data (`structural_edges` empty) | Skip centrality computation. Log info. Display "Centrality: N/A" in hotspots. |
| No entry points (`project_symbols` with `entrypoint_kind` empty) | Skip centrality computation. No entry points means no reachable paths. |
| Cycles in call graph (A calls B calls A) | BFS visited set prevents infinite loops. Cycle does not affect `entrypoints_reachable` count. |
| Very deep call chains (depth > 20) | BFS caps at 20 hops. Symbols beyond 20 hops are not counted as reachable from that entry point. |
| Very large call graphs (>100K edges) | Cap BFS at 50,000 reachable symbols per entry point. Log a warning if cap is hit. |
| Symbols reachable from many entry points (centrality > 20) | Apply up to 15 points within the Historical Hotspot category (max 30 points). Do not stack (no `15 * entrypoints_reachable`). |
| Symbols with 0 entry points reachable (dead code) | `entrypoints_reachable = 0`. No centrality risk weight. Still scored by other risk factors. |
| Multiple files defining the same symbol name | Disambiguate by `file_id`. Each `(symbol_id, file_id)` pair is a unique centrality entry. |
| Library crate (no `main` function) | Entry points are `HANDLER`-labeled symbols (from E1-4, `entrypoint_kind`). If no handlers exist either, centrality is skipped. |
| Incremental indexing | Centrality is recomputed from scratch on each `index` run. It is not incrementally updated. |

## 5. Acceptance Criteria

1. `changeguard index --analyze-graph` populates `symbol_centrality` when `structural_edges` and `project_symbols` entry points exist. A standard `changeguard index` run does NOT compute centrality.
2. A symbol reachable from 5+ entry points receives up to 15 points within the Historical Hotspot category (max 30 points) in `impact`.
3. `changeguard hotspots --centrality` includes centrality data in the output when `symbol_centrality` has data.
4. `changeguard hotspots` without `--centrality` produces output identical to the current version (no regression).
5. When `structural_edges` is empty, hotspots display "Centrality: N/A" and `impact` skips centrality risk scoring. Running `changeguard index` without `--analyze-graph` produces no centrality data.
6. Cycles in the call graph do not cause infinite loops or crashes.
7. BFS traversal caps at 20 hops from each entry point.

## 6. Verification Gate

- **Fixture test**: A function called by 5 route handlers gets `entrypoints_reachable = 5` in `symbol_centrality` (after running `changeguard index --analyze-graph`).
- **Fixture test**: A function not reachable from any entry point gets `entrypoints_reachable = 0`.
- **Hotspot test**: `changeguard hotspots --centrality` includes the centrality column when `symbol_centrality` has data.
- **Hotspot test**: `changeguard hotspots` without `--centrality` flag produces identical output to baseline (no extra column).
- **Impact test**: Changing a symbol with `entrypoints_reachable > 5` produces risk reason `"High centrality: {symbol} reachable from {N} entry points"` and up to 15 points within the Historical Hotspot category (max 30 points).
- **Cycle test**: A call graph with a cycle (A -> B -> A) does not cause infinite loop in centrality computation.
- **Empty-table test**: With no `structural_edges` data, `hotspots` shows "Centrality: N/A" and `impact` produces no centrality-related risk reasons.
- **Performance test**: Centrality computation for a call graph with 10,000 edges completes in under 5 seconds.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M16 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E2 tables for fixture repos
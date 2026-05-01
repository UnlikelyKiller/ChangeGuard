# Specification: Track E2-1 - Call Graph and Structural Coupling

## 1. Objective
Build a lightweight call graph from tree-sitter ASTs, mapping which symbols call which other symbols within the project. Store the result as edges in a new `structural_edges` table (Migration M16) and integrate the data into the `impact` risk-reason pipeline and the `verify` prediction pipeline. This is the first track of Phase E2 (Behavioral Mapping) and depends on the Phase E1 project index (`project_symbols` table from E1-1, entry points from E1-4).

## 2. Deliverables

### 2.1 Call Extraction via Tree-Sitter Queries
- **Target files**: `src/index/languages/rust.rs`, `src/index/languages/typescript.rs`, `src/index/languages/python.rs`
- **Details**: Add call-expression extraction to each language module. Each module must identify `call_expression` nodes and resolve the callee to a project-internal symbol where possible.
  - **Rust**: Extract `call_expression` nodes. Resolve the callee against `project_symbols` (functions, methods, trait methods). For method calls (`receiver.method()`), store the method name and mark `call_kind = 'METHOD_CALL'`. For trait-object dispatch, mark `call_kind = 'TRAIT_DISPATCH'`. For unresolved dynamic calls, mark `call_kind = 'DYNAMIC'`.
  - **TypeScript**: Extract `call_expression` and `new_expression` nodes. Resolve against `project_symbols`. For `new ClassName()`, store as `METHOD_CALL`. For callback invocations, mark `call_kind = 'DYNAMIC'`.
  - **Python**: Extract `call` nodes in function bodies. Resolve against `project_symbols`. For `obj.method()` patterns, mark `call_kind = 'METHOD_CALL'`. For `getattr()` or other dynamic dispatch, mark `call_kind = 'DYNAMIC'`.

### 2.2 Structural Edges Table (Migration M16)
- **Target file**: `src/state/migrations.rs`
- **Details**: Add Migration M16 creating the `structural_edges` table:
  ```sql
  CREATE TABLE IF NOT EXISTS structural_edges (
      id                  INTEGER PRIMARY KEY AUTOINCREMENT,
      caller_symbol_id    INTEGER NOT NULL REFERENCES project_symbols(id),
      caller_file_id      INTEGER NOT NULL REFERENCES project_files(id),
      callee_symbol_id    INTEGER REFERENCES project_symbols(id),
      callee_file_id      INTEGER REFERENCES project_files(id),
      unresolved_callee   TEXT,           -- non-NULL when callee is not in project_symbols
      call_kind           TEXT NOT NULL DEFAULT 'DIRECT',
      resolution_status   TEXT NOT NULL DEFAULT 'RESOLVED',  -- RESOLVED, AMBIGUOUS, UNRESOLVED, CAPPED
      confidence          REAL NOT NULL DEFAULT 1.0,
      evidence            TEXT,
      FOREIGN KEY (caller_symbol_id) REFERENCES project_symbols(id),
      FOREIGN KEY (caller_file_id) REFERENCES project_files(id),
      FOREIGN KEY (callee_symbol_id) REFERENCES project_symbols(id),
      FOREIGN KEY (callee_file_id) REFERENCES project_files(id)
  );
  CREATE INDEX IF NOT EXISTS idx_structural_edges_caller
      ON structural_edges(caller_symbol_id, caller_file_id);
  CREATE INDEX IF NOT EXISTS idx_structural_edges_callee
      ON structural_edges(callee_symbol_id, callee_file_id);
  ```
- The `call_kind` enum is stored as TEXT: `DIRECT`, `METHOD_CALL`, `TRAIT_DISPATCH`, `DYNAMIC`, `EXTERNAL`.
- The `resolution_status` enum is stored as TEXT: `RESOLVED` (callee found in project_symbols), `AMBIGUOUS` (multiple candidate matches), `UNRESOLVED` (callee name extracted but no project_symbols match), `CAPPED` (edge count capped per file).
- The `unresolved_callee` column stores the raw callee name when `callee_symbol_id` is NULL (i.e., when `resolution_status` is `UNRESOLVED` or `AMBIGUOUS`). It is NULL for fully resolved edges.
- The `confidence` column stores a value between 0.0 and 1.0 indicating extraction confidence. `DIRECT` and `METHOD_CALL` edges default to 1.0. `TRAIT_DISPATCH` edges default to 0.8. `DYNAMIC` edges default to 0.5. `EXTERNAL` edges default to 0.3.
- The `evidence` column stores an optional JSON string describing what was observed (e.g., `"call_expr:helper()"`, `"method_call:obj.process()"`).

### 2.3 Call Graph Builder Module
- **Target file**: New `src/index/call_graph.rs`
- **Details**: Implement `CallGraphBuilder` that:
  1. Queries `project_symbols` for all symbols in the project and `project_files` for all indexed files.
  2. Iterates over source files, dispatching to language-specific call extractors.
  3. For each extracted call, attempts to resolve the callee to a `project_symbols` entry (matching by symbol name + file path). Resolved callees produce `callee_symbol_id` and `callee_file_id` foreign keys.
  4. Resolved calls become `DIRECT`, `METHOD_CALL`, or `TRAIT_DISPATCH` edges with `resolution_status = 'RESOLVED'`.
  5. Unresolved calls store the raw callee name in `unresolved_callee`, leave `callee_symbol_id` as NULL, and set `resolution_status = 'UNRESOLVED'`. Ambiguous matches (multiple candidates) set `resolution_status = 'AMBIGUOUS'`.
  6. Cross-language calls (e.g., Python calling Rust via FFI) are marked `EXTERNAL` and stored for reference but excluded from downstream centrality computation.
  7. Recursive calls (caller == callee) are stored but not followed during traversal.
  8. Sets `confidence` based on `call_kind`: DIRECT/METHOD_CALL=1.0, TRAIT_DISPATCH=0.8, DYNAMIC=0.5, EXTERNAL=0.3.
  9. Populates `evidence` with a brief description of the observed call site (e.g., `"call_expr:helper()"`, `"method_call:obj.process()"`).
  10. Streams edges to SQLite in batches of 500 to avoid memory spikes on large repos.

### 2.4 Index Command Integration
- **Target file**: `src/commands/` (new `index.rs` or extending existing command structure)
- **Details**: The `changeguard index` command (established in E1-1) must invoke the `CallGraphBuilder` after `project_symbols` is populated. This is a sequential dependency: symbols must exist before edges can reference them.

### 2.5 Impact Integration
- **Target file**: `src/impact/analysis.rs`
- **Details**: When computing risk for a changed symbol, query `structural_edges` for all rows where `callee_symbol_id` matches the changed symbol's `project_symbols` row ID. For each matching caller, add a risk reason: `"Structurally coupled: {caller_symbol} calls {callee_symbol}"`. This supplements the existing risk reasons from imports, protected paths, and temporal coupling.
- **Risk weight**: Structural coupling contributes to the Historical Hotspot category (max 30 points). Symbols reachable from >5 entry points receive up to 15 points within this category.

### 2.6 Verify Integration
- **Target file**: `src/verify/predict.rs`
- **Details**: Add structural-edge-based prediction as an additional signal. When a symbol changes, query `structural_edges` for callers of the changed symbol (matching on `callee_symbol_id`). Add those callers as predicted verification targets with prediction reason `"Structural call from {caller_symbol}"`. This runs alongside (not replacing) existing import-based and temporal-coupling prediction.

## 3. Constraints & Guidelines
- **No performance regression**: The `impact` command must complete in under 5 seconds for 200 changed files. Structural edge queries must use the index on `callee_symbol_id/callee_file_id`. If the `structural_edges` table is empty (no prior `index` run), skip the query entirely.
- **Graceful degradation**: If `project_symbols` is empty or `structural_edges` is empty, `impact` and `verify` must still function normally, just without structural-coupling signals.
- **Deterministic over speculative**: Unresolved or dynamic calls must be labeled `DYNAMIC` or `EXTERNAL`, never as `DIRECT`.
- **Edge cap**: If a single file would produce more than 50,000 edges, cap at 50,000 and log a warning. Prioritize edges involving public symbols.
- **Backward-compatible schema**: Migration M16 is purely additive. No existing table is modified.
- **Single binary**: No new crate dependencies. Uses existing `tree-sitter`, `rusqlite`, and `serde`.

## 4. Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Dynamic dispatch (trait objects, function pointers, callbacks) | Mark as `DYNAMIC` with `resolution_status = 'UNRESOLVED'`. Do not attempt speculative resolution. |
| Cross-language calls (Python calling Rust via FFI) | Mark as `EXTERNAL`. Exclude from centrality computation. Set `confidence = 0.3`. |
| Recursive calls (function calls itself) | Store the edge. Do not follow recursively during BFS/centrality traversal. |
| Very large call graphs (>100K edges per file) | Cap at 50K edges per file. Set `resolution_status = 'CAPPED'` for overflow edges. Log warning. Prioritize public-symbol edges. |
| Generics/monomorphization | Store the generic call site. Do not store per-monomorphization edges. |
| Method calls on `self` | Store with `call_kind = 'METHOD_CALL'`. Resolve to the implementing type where possible. |
| Closures/lambda calls | Mark as `DYNAMIC` with `resolution_status = 'UNRESOLVED'`. The callee is the closure invocation pattern, not a named symbol. |
| Macro-generated calls | Skip calls inside macro bodies that cannot be resolved statically. Mark as `DYNAMIC` with `unresolved_callee` if the callee name is extractable. |
| Empty `project_symbols` table | Skip call graph construction entirely. Log info: "No project symbols indexed; skipping call graph." |
| File not parseable | Skip file. Continue to next. Accumulate warnings. |
| Same symbol called from multiple sites | Each call site produces a separate edge row. |
| Multiple candidate callees for one call | Set `resolution_status = 'AMBIGUOUS'`. Store `unresolved_callee` with the ambiguous name. Do not pick one candidate. |

## 5. Acceptance Criteria

1. `changeguard index` populates `structural_edges` for supported languages (Rust, TypeScript, Python).
2. `changeguard impact` includes structurally-coupled callers in `risk_reasons` when a changed symbol appears as a callee in `structural_edges`.
3. `changeguard verify` includes structurally-predicted files in verification plans when a changed symbol has callers in `structural_edges`.
4. Dynamic and unresolved calls are labeled `DYNAMIC` or `EXTERNAL`, never as `DIRECT`. Unresolved callees store raw name in `unresolved_callee` with `resolution_status = 'UNRESOLVED'`.
5. If `structural_edges` is empty, `impact` and `verify` produce identical output to the current version (no regression).
6. Performance: edge extraction for a 500-file repo completes in under 10 seconds. Edge lookup during `impact` adds less than 100ms.

## 6. Verification Gate

- **Fixture test**: A Rust project with `main()` calling `helper()` calling `internal()` produces edges `main -> helper` and `helper -> internal` in `structural_edges`.
- **Impact test**: Changing `internal()` produces a risk reason mentioning `helper` as a structurally-coupled caller.
- **Verify test**: Changing `internal()` predicts `helper` as a verification target with structural-call reasoning.
- **Multi-language test**: A TypeScript project with `app.get("/users", getUsers)` produces a call edge from the route registration to `getUsers`.
- **Empty-table test**: With no `structural_edges` data, `impact` and `verify` produce output identical to the baseline (no regressions).
- **Performance test**: Call graph extraction for a 500-file fixture repo completes in under 10 seconds.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M16 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E2 tables for fixture repos
# Track 50-2: Advanced Passive Doc Types (13+ formats)

## Objective

Extend the Track 50-1 document template engine with 15 specialized "passive" documentation types. Each type is backed by a CozoDB Datalog query template and rendered into deterministic Markdown or Mermaid.js. The system must be registry-driven, local-first, resilient to individual template failures, and produce byte-identical output for identical graph state.

## Context

Track 50-1 establishes `src/docs/generator.rs` and the `DocTemplate` trait for basic exports (dependency graph, symbol table, module summary). This track adds advanced types that leverage the full depth of the Knowledge Graph (KG), SQLite tables, and impact enrichment data. The resulting documents are "passive" — intended for human reading, AI agent context caching, and long-term architectural record keeping.

## Requirements

### Functional

1. **Extend Existing Registry**: Reuse the existing `DocRegistry` in `src/docs/generator.rs`. Add a `run_filtered()` method to support `--doc-type` filtering.
2. **New Templates**: Each doc type is a struct implementing the existing `DocTemplate` trait (which has `generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf>`). Each template internally runs its CozoDB query and formats output.
3. **CLI Integration**: Add `--doc-type <TYPE>` to `changeguard index --export-docs` (repeatable). If omitted, export all registered types.
4. **Output Directory**: All files written to `.changeguard/docs/` with deterministic filenames.
5. **Determinism**: All tabular output sorted by primary key; graph edges sorted lexicographically. Byte-identical output for identical graph state.
6. **Error Resilience**: A failure in a single doc type must not abort the entire export. Errors are logged via `tracing::warn` and collected in a per-type error summary.
7. **Doc Types to Implement**: At least 13 distinct types (15 listed below).

### Non-Functional

- **Local-first**: No network calls during generation.
- **Performance**: Generation of all types on the ChangeGuard repo must complete in <2 seconds.
- **Zero Regression**: Existing `cargo test`, `cargo clippy`, and `cargo fmt` pass unchanged.
- **Error Handling**: Use `thiserror` + `miette::Diagnostic`; no `unwrap()` or `expect()` in production code.
- **Module Boundaries**: `src/docs` owns doc generation. Queries are executed via `CozoStorage` in `src/state/storage_cozo.rs`; no direct `cozo::DbInstance` manipulation in `src/docs`.

## API Contracts

### `DocTemplate` Trait (from Track 50-1)

Use the EXISTING trait in `src/docs/generator.rs`:

```rust
pub trait DocTemplate: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf>;
}
```

### `DocRegistry` Extension

Extend the EXISTING `DocRegistry` in `src/docs/generator.rs`:

```rust
impl DocRegistry {
    pub fn run_filtered(
        &self,
        names: &[String],
        storage: &CozoStorage,
        output_dir: &Utf8Path,
    ) -> Result<Vec<Utf8PathBuf>>;
}
```

### CLI

Modify `src/commands/index.rs`:

- Add `doc_type: Option<Vec<String>>` to `IndexArgs`.
- When `--export-docs` is present, if `doc_type` is `Some`, call `registry.run_filtered()`, otherwise call `registry.run_all()`.

### Error Type

Reuse `DocGenerationError` from `src/docs/generator.rs`.

## Testing Strategy

| Test | Assertion |
|---|---|
| Registry list | `DocTypeRegistry::new().list()` contains all 15 names |
| Single type export | `render_doc` with synthetic CozoDB data returns valid Markdown |
| All-types export | `export_all` writes 15 files to a temp directory |
| Determinism | Two runs on identical graph produce identical file hashes |
| Error resilience | A broken query in one template does not abort `export_all` |
| CLI parsing | `--doc-type module_map --doc-type symbol_index` is parsed correctly |
| Mermaid syntax | Generated Mermaid files contain no syntax errors (basic regex checks) |
| Boundary check | `src/docs` does not import `cozo::DbInstance` directly |

## Dependencies & Risks

- **Depends on**: Track 50-1 (`src/docs/generator.rs`, `DocTemplate` trait, `--export-docs` flag).
- **CozoDB version lock**: Datalog syntax is tied to the `cozo` crate (0.7). Future upgrades may require query migration.
- **Performance risk**: Deep multi-hop queries on large graphs. Mitigation: cap hop depth at 2 for passive docs; use `:limit` where appropriate.
- **Schema drift**: New columns in `node`/`edge` relations. Mitigation: queries project only required columns.

## Success Criteria

- [ ] All 15 doc types (listed below) generate syntactically valid Markdown/Mermaid.
- [ ] `changeguard index --export-docs` populates `.changeguard/docs/` without panic.
- [ ] `changeguard index --export-docs --doc-type module_map` generates only `module_map.md`.
- [ ] `cargo test` passes with zero regressions.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` is clean.
- [ ] Output is deterministic (byte-identical across runs on same graph).

## Documentation Types (15)

1. **`module_map.md`** — File-level dependency graph (Mermaid flowchart) from `edge{relation: "depends_on" \| "imports"}`.
2. **`service_boundary.md`** — Service boundary map (Mermaid subgraph) from Louvain community detection on `edge`.
3. **`dependency_health.md`** — Health scorecard per module combining coupling depth, test coverage, and age.
4. **`symbol_index.md`** — Comprehensive table of `project_symbol` with kind, path, line, and visibility.
5. **`api_contract_index.md`** — Table of OpenAPI endpoints joined with KG service nodes.
6. **`change_hotspot_report.md`** — Ranked list of nodes by `risk_score` with 1-hop neighbor context.
7. **`semantic_neighbor_index.md`** — For each high-risk node, list semantic neighbors (`semantically_similar` edges).
8. **`data_flow_diagram.md`** — Mermaid diagram of route-handler ↔ data-model coupling.
9. **`test_coverage_gap.md`** — Symbols lacking recent test outcomes, cross-referenced with `test_outcome_history`.
10. **`adr_staleness_report.md`** — ADRs sorted by `staleness_days` and tier.
11. **`ci_pipeline_map.md`** — CI config files and their linked source blast-radius.
12. **`observability_signal_snapshot.md`** — Prometheus/log signals tied to services.
13. **`token_provenance_map.md`** — Symbol-level ledger attribution via `ledger_link`.
14. **`call_graph_detail.md`** — Per-file focused call graphs for high-complexity modules using existing `edge` relations.
15. **`federation_summary.md`** — Cross-repo ledger entries marked `[FEDERATED]`.

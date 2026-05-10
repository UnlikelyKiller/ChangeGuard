# Track 50-2 Plan: Advanced Passive Doc Types

## Goal

Implement 15 specialized documentation types backed by CozoDB Datalog queries and rendered into deterministic Markdown/Mermaid. Build on Track 50-1 (`src/docs/generator.rs`).

## Architecture

- `src/docs/error.rs`: Unified `DocError` using `thiserror` + `miette::Diagnostic`.
- `src/docs/registry.rs`: Central `DocTypeRegistry` for all templates.
- `src/docs/templates/structural.rs`: Module map, symbol index, service boundary.
- `src/docs/templates/behavioral.rs`: Hotspot report, semantic neighbor, dead code.
- `src/docs/templates/operational.rs`: API contract, data flow, test coverage gap.
- `src/docs/templates/governance.rs`: ADR staleness, CI pipeline, token provenance.
- `src/docs/templates/advanced.rs`: Dependency health, observability snapshot, federation summary.
- `src/commands/index.rs`: `--doc-type` CLI flag and wiring.

---

## Phase 1: Infrastructure (Registry, Errors, Generator Extension)

**Files:**
- Create: `src/docs/error.rs`
- Create: `src/docs/registry.rs`
- Modify: `src/docs/generator.rs` (assumed from Track 50-1)
- Modify: `src/docs/mod.rs`

### Task 1.1: Define `DocError`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_doc_error_display() {
        let e = DocError::TemplateNotFound("foo".into());
        assert_eq!(e.to_string(), "Template not found: foo");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package changeguard --lib docs::error::tests::test_doc_error_display -- --nocapture`

Expected: FAIL with "module `error` not found"

**Step 3: Write minimal implementation**

```rust
// src/docs/error.rs
use thiserror::Error;
use miette::Diagnostic;

#[derive(Debug, Error, Diagnostic)]
pub enum DocError {
    #[error("CozoDB query failed: {0}")]
    QueryFailed(#[from] miette::Report),
    #[error("IO error writing doc: {0}")]
    Io(#[from] std::io::Error),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
}
```

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git add src/docs/error.rs src/docs/mod.rs
git commit -m "feat(docs): add DocError enum for 50-2"
```

### Task 1.2: Implement `DocTypeRegistry`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_registry_lists_all_types() {
        let reg = DocTypeRegistry::new();
        assert!(reg.list().is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package changeguard --lib docs::registry::tests::test_registry_lists_all_types -- --nocapture`

Expected: FAIL with "module `registry` not found"

**Step 3: Write minimal implementation**

```rust
// src/docs/registry.rs
use std::collections::HashMap;
use crate::docs::generator::DocTemplate;

pub struct DocTypeRegistry {
    templates: HashMap<String, Box<dyn DocTemplate>>,
}

impl DocTypeRegistry {
    pub fn new() -> Self {
        Self { templates: HashMap::new() }
    }
    pub fn register<T: DocTemplate + 'static>(&mut self, template: T) {
        self.templates.insert(template.name().to_string(), Box::new(template));
    }
    pub fn get(&self, name: &str) -> Option<&dyn DocTemplate> {
        self.templates.get(name).map(|b| b.as_ref())
    }
    pub fn list(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.templates.keys().map(|s| s.as_str()).collect();
        keys.sort();
        keys
    }
    pub fn iter(&self) -> impl Iterator<Item = &dyn DocTemplate> + '_ {
        self.templates.values().map(|b| b.as_ref())
    }
}
```

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git add src/docs/registry.rs src/docs/mod.rs
git commit -m "feat(docs): add DocTypeRegistry"
```

### Task 1.3: Extend `DocGenerator` with `render_doc` and `export_all`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::storage_cozo::CozoStorage;
    use std::path::PathBuf;

    struct DummyTemplate;
    impl DocTemplate for DummyTemplate {
        fn name(&self) -> &'static str { "dummy" }
        fn filename(&self) -> &'static str { "dummy.md" }
        fn query(&self) -> &'static str { "?[id] <- [['a']]" }
        fn format(&self, rows: &cozo::NamedRows) -> Result<String, DocError> {
            Ok(format!("rows: {}", rows.rows.len()))
        }
    }

    #[test]
    fn test_render_doc_returns_content() {
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();
        let template = DummyTemplate;
        let content = render_doc(&cozo, &template).unwrap();
        assert_eq!(content, "rows: 1");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package changeguard --lib docs::generator::tests::test_render_doc_returns_content -- --nocapture`

Expected: FAIL with "function `render_doc` not found"

**Step 3: Write minimal implementation**

```rust
// src/docs/generator.rs
use camino::Utf8Path;
use crate::state::storage_cozo::CozoStorage;
use crate::docs::error::DocError;

pub fn render_doc(
    cozo: &CozoStorage,
    template: &dyn DocTemplate,
) -> Result<String, DocError> {
    let rows = cozo.run_script(template.query())
        .map_err(DocError::QueryFailed)?;
    template.format(&rows)
}

pub struct ExportedDoc {
    pub name: &'static str,
    pub path: camino::Utf8PathBuf,
    pub content: String,
}

pub fn export_all(
    cozo: &CozoStorage,
    registry: &DocTypeRegistry,
    out_dir: &Utf8Path,
) -> Result<Vec<ExportedDoc>, DocError> {
    use std::fs;
    let mut results = Vec::new();
    for template in registry.iter() {
        let content = match render_doc(cozo, template) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Doc type {} failed: {}", template.name(), e);
                continue;
            }
        };
        let file_path = out_dir.join(template.filename());
        fs::write(&file_path, &content).map_err(DocError::Io)?;
        results.push(ExportedDoc {
            name: template.name(),
            path: file_path,
            content,
        });
    }
    Ok(results)
}
```

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git add src/docs/generator.rs
git commit -m "feat(docs): add render_doc and export_all"
```

---

## Phase 2: Structural Doc Types

**Files:**
- Create: `src/docs/templates/structural.rs`
- Modify: `src/docs/registry.rs` (register templates in `new()` or via a builder)
- Modify: `src/docs/mod.rs`

### Task 2.1: `ModuleMapTemplate`

**Goal**: Mermaid flowchart of file-level `depends_on`/`imports` edges.

**Step 1: Write the failing test**

```rust
#[test]
fn test_module_map_mermaid_syntax() {
    let cozo = setup_test_cozo();
    cozo.run_script("?[source, target, relation, confidence, provenance_id] <- [['a.rs', 'b.rs', 'depends_on', 1.0, 'tx1']] :put edge").unwrap();
    let template = ModuleMapTemplate;
    let md = render_doc(&cozo, &template).unwrap();
    assert!(md.starts_with("```mermaid"));
    assert!(md.contains("flowchart TD"));
    assert!(md.contains("a_rs"));
    assert!(md.contains("b_rs"));
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL with "ModuleMapTemplate not found"

**Step 3: Write minimal implementation**

```rust
// src/docs/templates/structural.rs
pub struct ModuleMapTemplate;

impl DocTemplate for ModuleMapTemplate {
    fn name(&self) -> &'static str { "module_map" }
    fn filename(&self) -> &'static str { "module_map.md" }
    fn query(&self) -> &'static str {
        "?[source, target] := *edge{source, target, relation: 'depends_on'} \\n         | *edge{source, target, relation: 'imports'}"
    }
    fn format(&self, rows: &cozo::NamedRows) -> Result<String, DocError> {
        let mut out = String::from("# Module Map\n\n```mermaid\nflowchart TD\n");
        let mut edges: Vec<(String, String)> = Vec::new();
        for row in &rows.rows {
            if let (Some(cozo::DataValue::Str(s)), Some(cozo::DataValue::Str(t))) =
                (row.first(), row.get(1))
            {
                edges.push((sanitize_id(s), sanitize_id(t)));
            }
        }
        edges.sort();
        edges.dedup();
        for (s, t) in edges {
            out.push_str(&format!("    {} --> {}\n", s, t));
        }
        out.push_str("```\n");
        Ok(out)
    }
}

fn sanitize_id(s: &str) -> String {
    s.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
}
```

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git add src/docs/templates/structural.rs src/docs/mod.rs
git commit -m "feat(docs): add ModuleMapTemplate"
```

### Task 2.2: `SymbolIndexTemplate`

**Goal**: Markdown table of `project_symbol`.

**Step 1: Write the failing test**

```rust
#[test]
fn test_symbol_index_table() {
    let cozo = setup_test_cozo_with_symbols();
    let template = SymbolIndexTemplate;
    let md = render_doc(&cozo, &template).unwrap();
    assert!(md.contains("# Symbol Index"));
    assert!(md.contains("| Symbol | Kind | Path | Line | Public |"));
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL

**Step 3: Write minimal implementation**

Query:
```datalog
?[symbol_name, symbol_kind, file_path, line_start, is_public] := *project_symbol{symbol_name, symbol_kind, file_path, line_start, is_public}
```

Format as Markdown table sorted by `symbol_name`.

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git commit -m "feat(docs): add SymbolIndexTemplate"
```

### Task 2.3: `ServiceBoundaryTemplate`

**Goal**: Mermaid subgraph from Louvain communities.

**Step 1: Write the failing test**

```rust
#[test]
fn test_service_boundary_subgraph() {
    let cozo = setup_test_cozo_with_communities();
    let template = ServiceBoundaryTemplate;
    let md = render_doc(&cozo, &template).unwrap();
    assert!(md.contains("subgraph"));
    assert!(md.contains("ServiceBoundary"));
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL

**Step 3: Write minimal implementation**

Reuse `CozoStorage::run_community_louvain()` or equivalent Datalog:
```datalog
?[node, community_id] <~ CommunityDetectionLouvain(edges[src, dst])
edges[src, dst] := *edge{source: src, target: dst}
```

Group nodes by `community_id` and emit Mermaid subgraph blocks. Sort communities and nodes.

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git commit -m "feat(docs): add ServiceBoundaryTemplate"
```

---

## Phase 3: Behavioral Doc Types

**Files:**
- Create: `src/docs/templates/behavioral.rs`

### Task 3.1: `ChangeHotspotReportTemplate`

**Query:**
```datalog
?[id, label, risk_score] := *node{id, label, risk_score}, risk_score > 0.5
```
Sorted by `risk_score` descending.

**Format:** Markdown table + 1-hop neighbor list per hotspot.

Commit after test passes.

### Task 3.2: `SemanticNeighborIndexTemplate`

**Query:**
```datalog
?[source, neighbor] := *node{id: source, risk_score: rs}, rs > 0.5, *edge{source, target: neighbor, relation: 'semantically_similar'}
```

**Format:** Markdown table grouped by source node.

Commit after test passes.

### Task 3.3: `DeadCodeCandidateListTemplate`

**Query:**
```datalog
?[id, label] := *node{id, label, category: 'code'}, not *edge{target: id}
```

Cross-reference with git activity (SQLite `test_outcome_history` or `git log` via `crate::git` module) to filter symbols with low activity.

**Format:** Markdown table.

Commit after test passes.

---

## Phase 4: Operational Doc Types

**Files:**
- Create: `src/docs/templates/operational.rs`

### Task 4.1: `ApiContractIndexTemplate`

Join SQLite `api_endpoints` with KG `node` via path matching.

**Query (SQLite):**
```sql
SELECT path, method, summary, service_node FROM api_endpoints LEFT JOIN node ON ...
```

**Format:** Markdown table sorted by `path`.

Commit after test passes.

### Task 4.2: `DataFlowDiagramTemplate`

Query `edge{relation: 'data_flow'}` or derive from impact enrichment data.

**Format:** Mermaid diagram `graph LR` with route handlers on the left and data models on the right.

Commit after test passes.

### Task 4.3: `TestCoverageGapTemplate`

Query `project_symbol` left-outer-joined with `test_outcome_history` on `qualified_name`.

**Format:** Markdown table of symbols with no recent test outcome.

Commit after test passes.

---

## Phase 5: Governance Doc Types

**Files:**
- Create: `src/docs/templates/governance.rs`

### Task 5.1: `AdrStalenessReportTemplate`

Query SQLite for ledger entries or `relevant_decisions` with `staleness_days`.

**Format:** Markdown table with tier badges (`[STALE]`, `[AGING]`, `[FRESH]`).

Commit after test passes.

### Task 5.2: `CiPipelineMapTemplate`

Query `node{category: 'ci_config'}` and 1-hop `edge` to source files.

**Format:** Mermaid flowchart.

Commit after test passes.

### Task 5.3: `TokenProvenanceMapTemplate`

Query `ledger_link` joined with `project_symbol`.

**Format:** Markdown table: Symbol | Last Tx ID | Tx Date | Change Type.

Commit after test passes.

---

## Phase 6: Advanced Scoring & Federation

**Files:**
- Create: `src/docs/templates/advanced.rs`

### Task 6.1: `DependencyHealthTemplate`

Composite query:
- Coupling depth: in-degree + out-degree from `edge`.
- Test pass rate: from SQLite `test_outcome_history`.
- File age: from `node` metadata or git log.

Compute a 0-100 health score in Rust.

**Format:** Markdown table sorted by health ascending.

Commit after test passes.

### Task 6.2: `ObservabilitySignalSnapshotTemplate`

Query SQLite `observability_snapshots` joined with `node` service labels.

**Format:** Markdown table with signal type, value, threshold status.

Commit after test passes.

### Task 6.3: `FederationSummaryTemplate`

Query `ledger_entry` where `trace_id` starts with `SIBLING_` or origin is `FEDERATED`.

**Format:** Markdown table with `[FEDERATED]` prefix.

Commit after test passes.

---

## Phase 7: CLI Integration

**Files:**
- Modify: `src/commands/index.rs`
- Modify: `src/cli.rs` (if `IndexArgs` is constructed via CLI parser)

### Task 7.1: Add `--doc-type` flag

**Step 1: Write the failing test**

```rust
#[test]
fn test_parse_doc_type_flag() {
    let args = parse_index_args(&["--export-docs", "--doc-type", "module_map"]);
    assert_eq!(args.doc_type, Some(vec!["module_map".to_string()]));
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL with "field `doc_type` not found"

**Step 3: Write minimal implementation**

Add `doc_type: Option<Vec<String>>` to `IndexArgs` in `src/commands/index.rs` and wire the CLI parser.

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/index.rs src/cli.rs
git commit -m "feat(cli): add --doc-type flag to index command"
```

### Task 7.2: Wire export logic into `execute_index`

When `--export-docs` is present:
1. Build a `DocTypeRegistry` and register all 15 templates.
2. If `doc_type` is `Some`, filter `registry.iter()` to matching names.
3. Call `export_all(cozo, &registry, docs_dir)`.
4. Print summary of exported files and any errors.

**Step 1: Write the failing integration test**

```rust
#[test]
fn test_cli_export_docs_creates_files() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    // Initialize a minimal repo and CozoDB
    // Run execute_index with IndexArgs { export_docs: true, doc_type: None, ... }
    // Assert `.changeguard/docs/module_map.md` exists
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL

**Step 3: Write minimal implementation**

Implement the wiring in `execute_index`.

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git commit -m "feat(cli): wire doc export into execute_index"
```

---

## Phase 8: Final Validation

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test` — all existing and new tests pass.
- [ ] Run `changeguard index --export-docs` on the ChangeGuard repo and inspect `.changeguard/docs/`.
- [ ] Verify determinism: run export twice, `Get-FileHash` (or `md5sum`) of outputs must match.
- [ ] Update `conductor/conductor.md` to mark Track 50-2 as **Implemented**.
- [ ] `changeguard ledger commit <tx-id> --summary "Implement Track 50-2 advanced passive doc types" --reason "Complete 15 doc type templates with deterministic output"`

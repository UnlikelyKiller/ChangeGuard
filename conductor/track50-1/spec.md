# Specification: Track 50-1 — Document Template Engine & Basic Exports

## Objective

Implement a native documentation generation engine that queries the ChangeGuard Knowledge Graph (CozoDB) and exports structural data into standard Markdown and Mermaid.js formats. This track establishes the foundation for the 13+ passive documentation types planned for Milestone D.

## Context

ChangeGuard maintains a high-fidelity Knowledge Graph in CozoDB containing file nodes, symbol nodes, and call edges. Currently, this data is consumed only by internal impact analysis and semantic search. This track creates a "passive documentation" pipeline that renders graph data into human-readable artifacts suitable for GitHub, Obsidian, and AI agent context windows.

The engine must be trait-based and registry-driven so that future tracks (e.g., Track 50-2) can add new templates without modifying CLI wiring.

## Requirements

### Functional Requirements

1. **Template Engine (`src/docs/generator.rs`)**
   - Define a `DocTemplate` trait:
     ```rust
     pub trait DocTemplate: Send + Sync {
         fn name(&self) -> &'static str;
         fn description(&self) -> &'static str;
         fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> miette::Result<Utf8PathBuf>;
     }
     ```
   - Implement a `DocRegistry` that holds all available templates and can resolve by name.
   - The registry must be extensible: adding a new template type in future tracks requires only registering a new instance.

2. **Basic Export Templates**
   - **`DependencyGraphTemplate`**: Exports a Mermaid `graph TD` diagram of file-level dependencies inferred from symbol call edges.
     - Query: join `edge` with `project_symbol` (twice) to map symbol-level edges to file-level edges.
     - Deduplicate edges; skip self-loops.
     - Sanitize Mermaid node IDs to avoid leading digits and special characters.
     - Output: `.changeguard/docs/dependency_graph.md`.
   - **`SymbolTableTemplate`**: Exports a Markdown table of all indexed symbols.
     - Query: `project_symbol` relation.
     - Columns: Qualified Name, Symbol Name, Kind, File Path, Line Range, Public.
     - Group by file path for readability.
     - Cap at 10,000 rows with a truncation note.
     - Output: `.changeguard/docs/symbol_table.md`.
   - **`ModuleSummaryTemplate`**: Exports a high-level module overview.
     - Query: `node` relation where `category = "file"`.
     - Group files by parent directory (module).
     - List file count per module and inter-module edge count (re-using `DependencyGraphTemplate` logic).
     - Output: `.changeguard/docs/module_summary.md`.

3. **CLI Integration**
   - Add `--export-docs` flag to `Commands::Index` in `src/cli.rs`.
   - Add `export_docs: bool` to `IndexArgs` in `src/commands/index.rs`.
   - When `--export-docs` is set, `execute_index` calls the doc generator after indexing completes (ignored when `--check` is set).
   - Output directory: `.changeguard/docs/`. Create if missing via `Layout`.
   - Print progress: `Generating docs: {name} ... {path}`.

4. **Error Handling & Resilience**
   - If CozoDB is empty or missing, print a warning and return `Ok(())` — do not fail the index command.
   - If an individual template fails, log the error and continue with remaining templates.
   - All production paths use `miette::Result`; no `unwrap` or `expect`.

### Non-Functional Requirements

- **Performance**: Each template query must complete in <500ms on the ChangeGuard repo itself.
- **Determinism**: Output files must be byte-identical given the same graph state (sort all rows and edges).
- **Local-first**: No external services required.
- **Module boundaries**: `src/docs/` owns generation logic; `src/commands/` owns CLI wiring.

## API Contracts

### CLI Flags

```rust
/// Export structural documentation from the Knowledge Graph
#[arg(long)]
export_docs: bool,
```

### Internal Interfaces

```rust
// src/docs/generator.rs
pub trait DocTemplate: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn generate(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> miette::Result<Utf8PathBuf>;
}

pub struct DocRegistry {
    templates: Vec<Box<dyn DocTemplate>>,
}

impl DocRegistry {
    pub fn default_registry() -> Self;
    pub fn resolve(&self, name: &str) -> Option<&dyn DocTemplate>;
    pub fn run_all(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> miette::Result<Vec<Utf8PathBuf>>;
}
```

### CozoDB Query Contracts

- **File Dependencies**:
  ```cozo
  ?[source_file, target_file] := 
      *edge{source: src_sym, target: tgt_sym},
      *project_symbol{qualified_name: src_sym, file_path: source_file},
      *project_symbol{qualified_name: tgt_sym, file_path: target_file},
      source_file != target_file
  ```
- **Symbol Table**:
  ```cozo
  ?[qualified_name, symbol_name, symbol_kind, file_path, line_start, line_end, is_public] := 
      *project_symbol{id, qualified_name, symbol_name, symbol_kind, file_path, line_start, line_end, is_public}
  ```
- **Module Files**:
  ```cozo
  ?[file_path, label] := *node{id: file_path, label, category: 'file'}
  ```

## Testing Strategy

### Unit Tests (in `src/docs/generator.rs`)

| Test | Fixture | Assertion |
|---|---|---|
| `test_dependency_graph_empty` | Empty in-memory CozoDB | Returns empty Mermaid graph with no edges |
| `test_dependency_graph_two_files` | 2 file nodes, 2 symbols, 1 cross-file edge | Mermaid contains `file_a.rs --> file_b.rs` |
| `test_symbol_table_sorted` | 3 symbols across 2 files | Table rows sorted by `file_path`, then `line_start` |
| `test_module_summary_groups` | 3 files in `src/a.rs`, `src/b.rs`, `tests/t.rs` | Correct module group counts |
| `test_registry_resolve` | Default registry | `resolve("dependency_graph")` returns `Some` |
| `test_registry_run_all` | Default registry + in-memory CozoDB | Returns exactly 3 `Utf8PathBuf` values |

### Integration Tests

- `tests/doc_generation.rs`: Run `execute_index` with `--export-docs` on a fixture repo; assert all three `.md` files exist and are non-empty.

### Manual Verification

- Run `changeguard index --export-docs` on ChangeGuard itself.
- Verify `dependency_graph.md` renders in GitHub preview.
- Verify `symbol_table.md` contains `CozoStorage` and `DocTemplate`.

## Dependencies & Risks

| Dependency | Risk | Mitigation |
|---|---|---|
| CozoDB schema stability | `project_symbol` or `edge` schema may drift | Lock query contracts in this spec; validate schema version at runtime |
| No file-level edges in KG | Must infer from symbol edges + `project_symbol` join | Documented in query contracts; accept minor over-approximation |
| Large symbol tables | `symbol_table.md` may be huge on monorepos | Cap at 10k rows per table with a `... truncated` note |
| Windows path separators | Mermaid IDs and Markdown paths must use `/` | Normalize all paths to forward slashes before writing |

## Success Criteria

- `changeguard index --export-docs` generates three `.md` files in `.changeguard/docs/`.
- `dependency_graph.md` contains syntactically valid Mermaid `graph TD` with no self-loops.
- `symbol_table.md` contains a Markdown table with headers and at least one data row when the graph is non-empty.
- `module_summary.md` lists every directory that contains file nodes.
- All outputs are deterministic: same graph state produces identical file bytes.
- `cargo test` passes with zero regressions.

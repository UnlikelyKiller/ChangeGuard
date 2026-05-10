# Plan: Track 50-1 — Document Template Engine & Basic Exports

## Goal

Create a system for querying the CozoDB KG and exporting structural data to Markdown/Mermaid formats.

---

## Phase 1: Infrastructure & Trait Definitions (Red Commit)

- [ ] Task 1.1: Create `src/docs/generator.rs`.
- [ ] Task 1.2: Define `DocTemplate` trait with `name() -> &'static str`, `description() -> &'static str`, and `generate(storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Utf8PathBuf>`.
- [ ] Task 1.3: Define `DocRegistry` struct with `templates: Vec<Box<dyn DocTemplate>>`.
- [ ] Task 1.4: Implement `DocRegistry::default_registry()` returning a registry with placeholder templates that return `Ok(output_dir.join("{name}.md"))`.
- [ ] Task 1.5: Implement `DocRegistry::resolve(&self, name: &str) -> Option<&dyn DocTemplate>`.
- [ ] Task 1.6: Implement `DocRegistry::run_all(&self, storage: &CozoStorage, output_dir: &Utf8Path) -> Result<Vec<Utf8PathBuf>>`.
- [ ] Task 1.7: Add `pub mod generator;` to `src/docs/mod.rs`.
- [ ] Task 1.8: Add `const DOCS_DIR: &str = "docs";` and `docs_dir(&self) -> Utf8PathBuf` to `src/state/layout.rs`.
- [ ] Task 1.9: Update `Layout::ensure_state_dir()` to create `docs_dir()`.
- [ ] Task 1.10: Write unit test `test_registry_resolve` — assert `default_registry().resolve("dependency_graph")` returns `Some`.
- [ ] Task 1.11: Write unit test `test_registry_run_all_placeholders` — assert `run_all()` on an empty in-memory CozoDB returns 3 paths.
- [ ] Task 1.12: Run `cargo test --lib generator`; confirm placeholder tests pass (Green baseline for infrastructure).

---

## Phase 2: Datalog Query Layer & Data Models (Red → Green)

- [ ] Task 2.1: Define `FileDependency` struct in `src/docs/generator.rs`: `{ source_file: String, target_file: String }`.
- [ ] Task 2.2: Implement `query_file_dependencies(cozo: &CozoStorage) -> Result<Vec<FileDependency>>` using the Datalog query:
  ```cozo
  ?[source_file, target_file] := 
      *edge{source: src_sym, target: tgt_sym},
      *project_symbol{qualified_name: src_sym, file_path: source_file},
      *project_symbol{qualified_name: tgt_sym, file_path: target_file},
      source_file != target_file
  ```
- [ ] Task 2.3: Deduplicate results with `BTreeSet<(String, String)>` to guarantee determinism.
- [ ] Task 2.4: Define `SymbolRow` struct: `{ qualified_name: String, symbol_name: String, symbol_kind: String, file_path: String, line_start: i64, line_end: i64, is_public: bool }`.
- [ ] Task 2.5: Implement `query_symbol_table(cozo: &CozoStorage) -> Result<Vec<SymbolRow>>` using `project_symbol` query.
- [ ] Task 2.6: Sort results by `(file_path, line_start)` before returning.
- [ ] Task 2.7: Define `ModuleGroup` struct: `{ dir: String, files: Vec<String> }`.
- [ ] Task 2.8: Implement `query_module_groups(cozo: &CozoStorage) -> Result<Vec<ModuleGroup>>` by querying file nodes and grouping by parent directory in Rust.
- [ ] Task 2.9: Write unit test `test_dependency_graph_query` with in-memory CozoDB containing 2 files, 2 symbols, and 1 cross-file edge; assert exactly 1 `FileDependency` returned.
- [ ] Task 2.10: Write unit test `test_symbol_table_query` with 3 symbols; assert sorted order and correct fields.
- [ ] Task 2.11: Write unit test `test_module_groups_query` with 3 file nodes in different dirs; assert correct grouping.
- [ ] Task 2.12: Run `cargo test --lib generator`; confirm all new query tests pass (Green).

---

## Phase 3: Template Implementations (Red → Green)

- [ ] Task 3.1: Implement `DependencyGraphTemplate`.
  - Call `query_file_dependencies`.
  - Write Mermaid `graph TD` to `output_dir.join("dependency_graph.md")`.
  - Sanitize node IDs: replace non-alphanumeric with `_`; prefix with `f_` to avoid IDs starting with numbers.
  - Skip self-loops.
  - Sort edges lexicographically for determinism.
- [ ] Task 3.2: Implement `SymbolTableTemplate`.
  - Call `query_symbol_table`.
  - Write Markdown table to `output_dir.join("symbol_table.md")`.
  - Group with `### {file_path}` headings.
  - Cap rows at 10,000; append `> ... truncated` note if exceeded.
- [ ] Task 3.3: Implement `ModuleSummaryTemplate`.
  - Call `query_module_groups`.
  - Write Markdown list to `output_dir.join("module_summary.md")`.
  - Include inter-module edge count derived from `query_file_dependencies` (re-use deduped set).
- [ ] Task 3.4: Replace placeholder templates in `DocRegistry::default_registry()` with real implementations.
- [ ] Task 3.5: Write unit test `test_dependency_graph_mermaid_syntax` — assert output contains `graph TD` and `-->` but no self-loop syntax.
- [ ] Task 3.6: Write unit test `test_symbol_table_markdown_headers` — assert output contains `| Qualified Name |` header.
- [ ] Task 3.7: Write unit test `test_module_summary_lists_all_modules` — assert every directory appears in output.
- [ ] Task 3.8: Write unit test `test_deterministic_output` — run `generate()` twice on the same CozoDB; assert file bytes are identical.
- [ ] Task 3.9: Run `cargo test --lib generator`; confirm all template tests pass (Green).

---

## Phase 4: CLI Integration

- [ ] Task 4.1: Add `export_docs: bool` to `IndexArgs` in `src/commands/index.rs`.
- [ ] Task 4.2: Add `--export-docs` flag to `Commands::Index` in `src/cli.rs`.
- [ ] Task 4.3: Wire the new flag through the CLI dispatch in `src/cli.rs` (`Commands::Index { ..., export_docs }`).
- [ ] Task 4.4: In `execute_index()` in `src/commands/index.rs`, after successful indexing (and after the early-return branches for `--docs`, `--contracts`, `--semantic`, `--scip`), check `args.export_docs`.
- [ ] Task 4.5: If `args.export_docs`:
  - Skip if `args.check` is true (export requires fresh data).
  - Resolve `cozo_path = layout.state_subdir().join("ledger.cozo")`.
  - Open `CozoStorage::new(cozo_path.as_std_path())`.
  - If `node_count()? == 0`, print `Warning: Knowledge Graph is empty, skipping doc export.` and return `Ok(())`.
  - Ensure `layout.docs_dir()` exists.
  - Create `DocRegistry::default_registry()` and call `run_all()`.
  - Print each generated file path as `Doc: {path}`.
- [ ] Task 4.6: Write integration test `tests/doc_generation.rs`:
  - Create temp repo with 2 Rust files and a `.changeguard/state/ledger.cozo`.
  - Populate CozoDB with minimal `node`, `edge`, and `project_symbol` rows.
  - Run `execute_index(IndexArgs { export_docs: true, ..Default::default() })`.
  - Assert `.changeguard/docs/dependency_graph.md`, `.changeguard/docs/symbol_table.md`, `.changeguard/docs/module_summary.md` exist and are non-empty.
- [ ] Task 4.7: Run `cargo test --test doc_generation`; confirm integration test passes.

---

## Phase 5: Error Handling & Edge Cases

- [ ] Task 5.1: Add `DocGenerationError` enum in `src/docs/generator.rs` using `thiserror` and `miette::Diagnostic`.
  - Variants:
    - `#[error("CozoDB query failed: {0}")]` `QueryFailed(String)`
    - `#[error("I/O error writing doc output")]` `IoFailed(#[from] std::io::Error)`
    - `#[error("Knowledge Graph unavailable")]` `CozoUnavailable`
- [ ] Task 5.2: Ensure all `generate()` methods propagate errors via `?` and never use `unwrap`/`expect` in production paths.
- [ ] Task 5.3: In `DocRegistry::run_all()`, catch individual template errors with `match`, log via `tracing::warn!("Template '{}' failed: {:#}", name, err)`, and continue with remaining templates.
- [ ] Task 5.4: Normalize all file paths to forward slashes (`replace('\', "/")`) before writing to Markdown/Mermaid content.
- [ ] Task 5.5: Handle empty query results gracefully: templates must still produce valid (if minimal) output files.
- [ ] Task 5.6: Run `cargo clippy --all-targets --all-features -- -D warnings` and fix any issues.
- [ ] Task 5.7: Run `cargo fmt --all -- --check` and fix any issues.

---

## Phase 6: Final Validation

- [ ] Task 6.1: Run `cargo test --lib` — all new unit tests pass.
- [ ] Task 6.2: Run `cargo test --test '*'` — all integration tests pass.
- [ ] Task 6.3: Run full `cargo test` — zero regressions.
- [ ] Task 6.4: Manually run `changeguard index --export-docs` on the ChangeGuard repo.
- [ ] Task 6.5: Inspect `.changeguard/docs/dependency_graph.md` in GitHub preview; verify Mermaid renders.
- [ ] Task 6.6: Inspect `.changeguard/docs/symbol_table.md`; verify table formatting.
- [ ] Task 6.7: Inspect `.changeguard/docs/module_summary.md`; verify module groupings.
- [ ] Task 6.8: Run `changeguard verify` (or `cargo test` if verify aliases are unavailable) and confirm clean pass.

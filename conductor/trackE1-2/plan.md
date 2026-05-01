## Plan: Track E1-2 README and Documentation Ingestion

### Phase 1: Dependency and Schema
- [ ] Task 1.1: Add `pulldown-cmark = { version = "0.13.3", default-features = false }` to `Cargo.toml` dependencies.
- [ ] Task 1.2: Add `project_docs` table creation to migration M15 in `src/state/migrations.rs`. Table columns: `id INTEGER PRIMARY KEY AUTOINCREMENT`, `file_id INTEGER NOT NULL REFERENCES project_files(id)`, `title TEXT`, `summary TEXT`, `sections JSON`, `code_blocks JSON`, `internal_links JSON`, `confidence REAL NOT NULL DEFAULT 1.0`, `last_indexed_at TEXT NOT NULL`. Include index on `file_id`. E1-1 owns M15; coordinate with E1-1 for the shared migration.
- [ ] Task 1.3: Update `test_all_tables_exist` to verify `project_docs` is created.
- [ ] Task 1.4: Write integration test `test_insert_and_query_project_docs` that inserts a doc row and queries it back.

### Phase 2: Domain Types
- [ ] Task 2.1: Define `ParsedDoc` struct in `src/index/docs.rs` with fields: `file_path`, `title` (Option<String>), `summary` (Option<String>), `sections` (Vec<DocSection>), `code_blocks` (Vec<CodeBlock>), `internal_links` (Vec<InternalLink>).
- [ ] Task 2.2: Define `DocSection` struct with fields: `title` (String), `level` (u8), `line_start` (usize). Define `CodeBlock` struct with fields: `language` (Option<String>), `content_preview` (String, max 200 chars), `line_start` (usize), `line_end` (usize). Define `InternalLink` struct with fields: `target` (String), `line_start` (usize).
- [ ] Task 2.3: Define `DocIndexStats` struct with fields: `docs_indexed`, `parse_failures`, `missing_readme` (bool).
- [ ] Task 2.4: Add `pub mod docs;` to `src/index/mod.rs`.

### Phase 3: Markdown Parser
- [ ] Task 3.1: Implement `parse_markdown(content: &str, file_path: &str) -> ParsedDoc` in `src/index/docs.rs` using `pulldown_cmark::Parser`. Walk the AST and extract: title from first ATX heading level 1; sections from all headings; code blocks with language and content preview; internal links from inline link events.
- [ ] Task 3.2: Implement `extract_title(content: &str) -> Option<String>` that returns the first `# Heading` text. If no level-1 heading, return `None`.
- [ ] Task 3.3: Implement `extract_summary(content: &str, max_lines: usize, max_chars: usize) -> Option<String>` that concatenates paragraph text from the first `max_lines` lines, strips Markdown formatting, and truncates to `max_chars` (5,000). Returns `None` for empty documents.
- [ ] Task 3.4: Implement `extract_internal_links(content: &str) -> Vec<String>` that finds `[text](relative.md)` patterns and filters to local `.md` paths only.
- [ ] Task 3.5: Write unit tests for `parse_markdown` with fixtures: valid Markdown with headings/code blocks/links, malformed Markdown, empty file, very large file (simulate >100KB truncation), file with no heading.
- [ ] Task 3.6: Write unit tests for `extract_summary` verifying truncation at 5,000 characters and 500 lines.

### Phase 4: Doc Discovery and Indexing
- [ ] Task 4.1: Implement `ProjectIndexer::discover_doc_files(&self) -> Result<Vec<Utf8PathBuf>>` that checks for `README.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md` at the repo root, and follows one level of internal links from `README.md`.
- [ ] Task 4.2: Implement `ProjectIndexer::index_docs(&self) -> Result<DocIndexStats>` that discovers doc files, parses each with `parse_markdown`, inserts/updates `project_docs` rows, and returns stats.
- [ ] Task 4.3: Handle missing `README.md` by setting `missing_readme = true` in `DocIndexStats` (warning, not error).
- [ ] Task 4.4: Handle parse failures by inserting a row with `title` from the filename and `summary = NULL`, incrementing `parse_failures`.
- [ ] Task 4.5: Write integration tests for `discover_doc_files` and `index_docs` using a temp directory with fixture Markdown files.

### Phase 5: CLI Integration
- [ ] Task 5.1: Modify `execute_index` in `src/commands/index.rs` to call `ProjectIndexer::index_docs()` after source file indexing. Include `DocIndexStats` in the output.
- [ ] Task 5.2: Print doc indexing stats in the human-readable output (e.g., "Documentation: 3 files indexed, 0 failures, README: found").
- [ ] Task 5.3: Write CLI integration test for `changeguard index` on a repo with `README.md`, verifying `project_docs` is populated.

### Phase 6: Ask Command Integration
- [ ] Task 6.1: In `src/commands/ask.rs`, after constructing the system prompt, query `project_docs` for the README summary. If found, prepend a `## Project Context\nProject: {title}\n{summary}` section.
- [ ] Task 6.2: Truncate the project summary to fit within the existing token budget. Use the `truncate_for_context` function already in use.
- [ ] Task 6.3: If `project_docs` is empty, skip the project context section entirely (graceful degradation).
- [ ] Task 6.4: Write test verifying that `ask` includes project context when `project_docs` has data and omits it when empty.

### Phase 7: Audit/Doctor Integration
- [ ] Task 7.1: In the `audit` or `doctor` command, add a check for `project_docs` entries. If no README entry exists, emit warning: "No README.md found. Project documentation is missing."
- [ ] Task 7.2: If README exists but `summary` is NULL, emit: "README.md exists but could not be parsed."
- [ ] Task 7.3: Write test verifying audit warns about missing README and about unparseable README.
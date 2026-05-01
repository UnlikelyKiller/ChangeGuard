# Specification: Track E1-2 README and Documentation Ingestion

## Overview

This track adds the ability to parse `README.md` and other documentation files to ground ChangeGuard's analysis in the project's stated mission and structure. It uses the `pulldown-cmark` crate to extract structured content from Markdown files, storing the results in a new `project_docs` table (Migration M15, shared with E1-1). The parsed documentation is integrated with the `ask` command (included in Gemini system prompts) and the `audit` command (surfacing "No README found" as a health warning).

This track depends on E1-1 for the `index` command infrastructure and the `ProjectIndexer` pattern.

## Components

### 1. Database Migration M15 - Part B: `project_docs` Table (`src/state/migrations.rs`)

Add the `project_docs` table to the same M15 migration that E1-1 adds `project_files`, `index_metadata`, and `project_symbols`. **E1-1 owns M15**; this track adds its table to that shared migration.

```sql
CREATE TABLE IF NOT EXISTS project_docs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id         INTEGER NOT NULL REFERENCES project_files(id),
    title           TEXT,
    summary         TEXT,
    sections        JSON,
    code_blocks     JSON,
    internal_links  JSON,
    confidence      REAL NOT NULL DEFAULT 1.0,
    last_indexed_at TEXT NOT NULL,
    UNIQUE(file_id)
);
CREATE INDEX IF NOT EXISTS idx_project_docs_file_id
    ON project_docs(file_id);
```

- `file_id`: Foreign key to `project_files(id)`. Per expansion plan constraint #7, no downstream table may rely solely on `file_path + symbol_name` for joins. Use integer foreign keys.
- `title`: The first `# Heading` or `<title>` extracted from the document. Nullable because some files have no top-level heading.
- `summary`: Concatenated text of the first 500 lines (or entire document if shorter), stripped of Markdown formatting. Nullable because some files may be empty.
- `sections`: JSON array of `{title, level, line_start}` objects from all ATX headings. Stored as structured data for downstream consumption.
- `code_blocks`: JSON array of `{language, line_start, line_end}` objects. Stored as structured data for downstream consumption.
- `internal_links`: JSON array of `{target, line_start}` objects for relative Markdown link targets. Stored as structured data for downstream consumption.
- `confidence`: Float between 0.0 and 1.0. Per the expansion plan, every extracted fact must carry a confidence score.
- `file_id` is `UNIQUE` because each documentation file is indexed once.

### 2. Markdown Parser (`src/index/docs.rs`)

New module that parses Markdown files using `pulldown-cmark`.

**Target files (in priority order):**
1. `README.md` (repo root)
2. `CONTRIBUTING.md` (repo root)
3. `ARCHITECTURE.md` (repo root or `docs/`)
4. Files referenced from `README.md` via `[text](relative/path.md)` links (one level deep only)

**Extracted data:**
- **Title**: The first `# Heading` (ATX heading) or `<h1>` (HTML). If no top-level heading, the filename stem (e.g., "README").
- **Sections**: A list of `(heading_level, heading_text)` pairs from all ATX headings (level 1-6).
- **Code blocks**: A list of `(language, content_preview)` tuples. `language` is the info string (e.g., "rust", "typescript"). `content_preview` is the first 200 characters of the code block content.
- **Internal links**: A list of relative Markdown link targets extracted from `[text](path.md)` patterns. Only relative paths ending in `.md` are collected (absolute URLs and image links are excluded).
- **Summary**: The concatenated text of paragraphs in the first 500 lines, stripped of Markdown formatting.

**Key types:**
```rust
pub struct ParsedDoc {
    pub file_path: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub sections: Vec<DocSection>,        // stored as JSON in project_docs
    pub code_blocks: Vec<CodeBlock>,      // stored as JSON in project_docs
    pub internal_links: Vec<InternalLink>, // stored as JSON in project_docs
}

pub struct DocSection {
    pub title: String,
    pub level: u8,
    pub line_start: usize,
}

pub struct CodeBlock {
    pub language: Option<String>,
    pub content_preview: String,
    pub line_start: usize,
    pub line_end: usize,
}

pub struct InternalLink {
    pub target: String,
    pub line_start: usize,
}
```

**Parsing implementation:**
- Use `pulldown_cmark::Parser` to walk the Markdown AST.
- Extract `Heading(level, fragments)` events for sections and title.
- Extract `CodeBlock(language)` and `Text` events following `Start(CodeBlock)` for code blocks.
- Extract `Start(Link(url, ...))` events for internal links.
- Truncate summary at 5,000 characters (prevents unbounded storage).

### 3. Doc Indexing Integration (`src/index/project_index.rs`)

Extend `ProjectIndexer` with documentation indexing methods:

```rust
pub fn index_docs(&self) -> Result<DocIndexStats>
pub fn discover_doc_files(&self) -> Result<Vec<Utf8PathBuf>>
```

- `discover_doc_files()`: Walk the repo looking for `README.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md`, and any `.md` files referenced from `README.md`. Return the list.
- `index_docs()`: For each discovered file, parse with the Markdown parser, extract `title`, `summary`, `sections`, `code_blocks`, and `internal_links`, and insert/update `project_docs` (referencing `project_files(id)` via `file_id`). Files not found are skipped (not an error).

**`DocIndexStats`:**
```rust
pub struct DocIndexStats {
    pub docs_indexed: usize,
    pub parse_failures: usize,
    pub missing_readme: bool,
}
```

### 4. `index` Command Integration (`src/commands/index.rs`)

Extend the `execute_index` function to include documentation indexing as part of the `index` pipeline:
- After indexing source files, call `ProjectIndexer::index_docs()`.
- If `README.md` is not found, include `"No README.md found"` in the stats output (warning, not error).
- Print doc indexing stats alongside module/symbol stats.

### 5. `ask` Command Integration (`src/commands/ask.rs`)

When constructing the Gemini system prompt for the `ask` command:
- Query `project_docs` for the README's `summary` field.
- If available, prepend a `## Project Context` section to the system prompt containing: `Project: {title}\n{summary}`.
- Truncate the summary to fit within the token budget (respecting the existing `truncate_for_context` logic).
- If `project_docs` is empty or has no README entry, proceed without project context (graceful degradation).

### 6. `audit` Command Integration

In the `audit` or `doctor` command output:
- Check `project_docs` for a README entry.
- If no README exists, emit a project health warning: `"No README.md found. Project documentation is missing."`
- If README exists but `summary` is empty or `PARSE_FAILED`, emit: `"README.md exists but could not be parsed."`
- This is informational only; it does not block any operation.

## Constraints

- **New dependency:** `pulldown-cmark` version 0.13.3 (pure Rust, no native dependencies). Must be added to `Cargo.toml` with `default-features = false` to minimize compile time.
- **Graceful degradation:** If `pulldown-cmark` fails to parse a file, extract whatever is possible (title from filename, empty summary) and continue. Never crash on malformed Markdown.
- **No network access:** Documentation files are read from the local filesystem only. No URLs are fetched.
- **No `.env` reading:** This track does not read actual `.env` files. Only `.md` files are processed.
- **Token budget awareness:** When including project context in `ask` prompts, the summary must be truncated to fit within the existing token budget. Do not exceed the configured context window.

## Edge Cases

- **No README:** The most common case for new or minimal repos. Emit a warning. Do not create a `project_docs` row. Continue indexing other doc files if they exist.
- **Malformed Markdown:** `pulldown-cmark` is extremely tolerant and will parse most malformed Markdown. For truly broken content, extract the title from the filename and leave `summary` as `NULL`.
- **Very large README (>100KB):** Read only the first 500 lines for the summary. Store the full title (from first heading) regardless of file size.
- **Binary/encoded content in code blocks:** If a code block contains non-UTF-8 content, skip that code block. `pulldown-cmark` operates on Rust strings, so this should not occur for valid UTF-8 Markdown.
- **Circular links:** If `README.md` links to `A.md` which links back to `README.md`, follow only one level deep. Do not recurse into referenced files of referenced files.
- **Missing referenced files:** If `README.md` links to `docs/API.md` but that file does not exist, skip it. Do not create a `project_docs` row for a missing file.
- **Multiple READMEs (subdirectories):** Only index the `README.md` at the repository root. Subdirectory READMEs are a future enhancement.
- **CONTRIBUTING.md and ARCHITECTURE.md missing:** Skip without warning. These are optional documentation files.

## Acceptance Criteria

1. `changeguard index` populates `project_docs` when `README.md` exists in the repo root.
2. `changeguard index` does not crash or error when `README.md` is absent. It emits a warning.
3. `changeguard ask --narrative` includes a `## Project Context` section in the system prompt when `project_docs` has a README entry.
4. `changeguard ask` works normally (without project context) when `project_docs` is empty.
5. Parsed doc data includes `title`, `summary`, sections, code blocks, and internal links.
6. The `project_docs.summary` field is truncated to 5,000 characters maximum.
7. Files larger than 100KB have only their first 500 lines processed for the summary.
8. `pulldown-cmark` is added as a dependency in `Cargo.toml`.

## Verification Gate

- **Unit tests:** Markdown parser correctly extracts title, sections, code blocks, and internal links from a fixture Markdown file.
- **Unit tests:** Malformed Markdown produces a best-effort parse (title from filename, partial summary).
- **Unit tests:** Missing README triggers a warning without crashing.
- **Unit tests:** Very large Markdown file (>100KB) is truncated at 500 lines.
- **Integration test:** `changeguard index` on a fixture repo with `README.md`, `CONTRIBUTING.md`, and `ARCHITECTURE.md` populates `project_docs` with all three files.
- **Integration test:** `changeguard index` on a fixture repo without `README.md` emits a warning and continues.
- **Integration test:** `changeguard ask --narrative` on an indexed repo includes project summary in the system prompt.
- **Regression test:** Existing `ask` tests pass without `project_docs` data.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M15 applied cleanly to existing ledger.db
- [ ] `changeguard index` on a fixture repo produces non-empty project_symbols
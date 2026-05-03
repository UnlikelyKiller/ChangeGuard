# Specification: Track M2-1 — Document Crawler & Chunker

## Objective
Implement the document indexing pipeline: walk configured documentation paths, split files into semantic chunks, and store them in `doc_chunks`. This is the input stage for the retrieval pipeline built in M2-2.

## Components

### 1. `DocChunk` Type

```rust
pub struct DocChunk {
    pub file_path: String,
    pub chunk_index: usize,
    pub heading: Option<String>,
    pub content: String,
    pub token_count: usize,
}
```

### 2. Chunker (`src/docs/chunker.rs`)

**`chunk_markdown(content: &str, file_path: &str, chunk_tokens: usize, overlap_tokens: usize) -> Vec<DocChunk>`**

Algorithm:
1. Split the content on `\n## ` and `\n### ` heading boundaries.
2. For each section, record the heading text (the line immediately following the `##`/`###` marker).
3. If a section's `token_estimate` exceeds `chunk_tokens`, split further at `\n\n` (paragraph) boundaries.
4. Discard any resulting chunk where `token_count < 50`.
5. Assign sequential `chunk_index` values across the whole file.
6. `overlap_tokens` is reserved for future use; the chunker stores it but does not implement overlap in this track.

Determinism requirement: same `content` input always produces the same output regardless of call ordering.

### 3. Crawler (`src/docs/crawler.rs`)

**`crawl_docs(repo_root: &Utf8Path, include_globs: &[&str]) -> Result<Vec<DocFile>>`**

```rust
pub struct DocFile {
    pub path: Utf8PathBuf,
    pub content: String,
    pub last_modified: SystemTime,
}
```

- Uses existing gitignore-respecting walk logic (same approach as `changeguard index`).
- Accepts glob patterns relative to `repo_root`.
- Reads only files with extensions: `.md`, `.txt`, `.rst`, `.adoc`.
- Skips unreadable files with `tracing::warn!`; does not abort.
- Returns files in sorted order by path (deterministic).

### 4. Index Orchestrator (`src/docs/index.rs`)

**`run_docs_index(config: &Config, conn: &Connection) -> Result<DocsIndexSummary>`**

```rust
pub struct DocsIndexSummary {
    pub files_crawled: usize,
    pub chunks_new: usize,
    pub chunks_updated: usize,
    pub chunks_deleted: usize,
}
```

For each crawled file:
1. Chunk it.
2. For each chunk, check `doc_chunks` for existing row at `(file_path, chunk_index)`.
3. Compute `blake3(chunk.content)`. If hash matches stored row: skip.
4. If no existing row: INSERT → increment `chunks_new`. Call `embed_and_store`.
5. If hash differs: UPDATE content, heading, token_count → increment `chunks_updated`. Call `embed_and_store`.

After processing all crawled files:
6. DELETE `doc_chunks` rows whose `file_path` is not in the crawled set → increment `chunks_deleted`.
7. Also delete corresponding `embeddings` rows for those orphaned chunks.

When `config.local_model.base_url` is empty:
- Store chunks in `doc_chunks` normally.
- Skip `embed_and_store` calls silently.

### 5. CLI Integration

Add `--docs` flag to `IndexArgs` in `src/cli.rs`. When set, `execute_index()` calls `run_docs_index()` and prints:

```
Docs indexed: 4 files, 12 new chunks, 0 updated, 0 deleted.
```

When `config.docs.include` is empty:
```
No doc paths configured in [docs].include — skipping doc index.
```

## Test Specifications

### Chunker Tests (unit, fixture-based)
| Test | Assertion |
|---|---|
| 3 `##` sections | 3 chunks, headings populated |
| Section > chunk_tokens | Split at paragraph boundary, each under token cap |
| Section < 50 tokens | Discarded |
| No headings | Single chunk with `heading = None` |
| Same input twice | Identical output (determinism) |

### Crawler Tests (unit, tempdir)
| Test | Assertion |
|---|---|
| `.md` files found, `.rs` skipped | Only .md in result |
| `.gitignore`-covered path skipped | Not in result |
| Unreadable file | Skipped with warn; result still returned |
| Returns sorted by path | Paths in ascending order |

### Index Tests (integration, tempdir + mock embed server)
| Test | Assertion |
|---|---|
| Fresh index 2 files | `chunks_new = N`, `chunks_updated = 0`, `chunks_deleted = 0` |
| Re-index unchanged | `chunks_new = 0`, `chunks_updated = 0` |
| Edit one section | `chunks_updated = 1` |
| Delete file, re-index | Chunk rows removed; `chunks_deleted > 0` |
| `base_url = ""` | Index completes; no HTTP calls; chunks stored |

## Constraints & Guidelines

- **TDD**: Each test must fail before the implementation exists.
- **No unwraps in production paths**: crawler and indexer use `Result` propagation.
- **Path normalization**: `file_path` stored in `doc_chunks` always uses forward slashes, relative to repo root.
- **Test isolation**: Each test uses its own `tempfile::tempdir()` SQLite path.
- **Existing gitignore logic**: reuse, do not re-implement.

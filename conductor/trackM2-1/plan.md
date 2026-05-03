## Plan: Track M2-1 — Document Crawler & Chunker

### Phase 1: Document Chunker
- [ ] Task 1.1: Create `src/docs/chunker.rs` with `chunk_markdown(content: &str, file_path: &str, chunk_tokens: usize, overlap_tokens: usize) -> Vec<DocChunk>`.
- [ ] Task 1.2: `DocChunk` struct: `file_path: String`, `chunk_index: usize`, `heading: Option<String>`, `content: String`, `token_count: usize`.
- [ ] Task 1.3: Split on `##` and `###` headings; preserve heading text as `DocChunk.heading`.
- [ ] Task 1.4: If a section exceeds `chunk_tokens`, split on paragraph boundaries (double newline).
- [ ] Task 1.5: Discard chunks with `token_count < 50`.
- [ ] Task 1.6: Chunking is deterministic: same input always produces the same `Vec<DocChunk>`.
- [ ] Task 1.7: Write unit test: markdown with 3 `##` sections produces 3 chunks with correct headings.
- [ ] Task 1.8: Write unit test: a section exceeding `chunk_tokens` is split at paragraph boundary.
- [ ] Task 1.9: Write unit test: a section under 50 tokens is discarded.
- [ ] Task 1.10: Write unit test: file with no headings is treated as one chunk.
- [ ] Task 1.11: Write unit test: identical input called twice produces identical output (determinism).

### Phase 2: Document Crawler
- [ ] Task 2.1: Create `src/docs/crawler.rs` with `crawl_docs(repo_root: &Utf8Path, include_globs: &[&str]) -> Result<Vec<DocFile>>`.
- [ ] Task 2.2: `DocFile` struct: `path: Utf8PathBuf`, `content: String`, `last_modified: SystemTime`.
- [ ] Task 2.3: Respect `.gitignore` rules via the existing gitignore logic in the codebase.
- [ ] Task 2.4: Only read files with extensions: `.md`, `.txt`, `.rst`, `.adoc`; skip everything else silently.
- [ ] Task 2.5: Skip unreadable files with a `WARN` log entry; do not abort.
- [ ] Task 2.6: Write unit test: crawler finds `.md` files matching globs but skips `.rs` files.
- [ ] Task 2.7: Write unit test: crawler skips files in a path covered by a `.gitignore` rule.
- [ ] Task 2.8: Write unit test: an unreadable file (permissions denied — simulate by using a nonexistent path) is skipped without error.

### Phase 3: Document Index Orchestration
- [ ] Task 3.1: Create `src/docs/index.rs` with `run_docs_index(config: &Config, conn: &Connection) -> Result<DocsIndexSummary>`.
- [ ] Task 3.2: `DocsIndexSummary`: `files_crawled: usize`, `chunks_new: usize`, `chunks_updated: usize`, `chunks_deleted: usize`.
- [ ] Task 3.3: For each crawled file, chunk it, compare each chunk's `blake3(content)` against `doc_chunks` table.
- [ ] Task 3.4: Insert new chunks, update changed chunks, delete orphaned chunk rows (file removed or section deleted).
- [ ] Task 3.5: For each new or updated chunk, call `embed_and_store` with `entity_type = "doc_chunk"` and `entity_id = "{file_path}::{chunk_index}"`.
- [ ] Task 3.6: When `config.local_model.base_url` is empty, store chunks in `doc_chunks` but skip embedding (graceful degradation).
- [ ] Task 3.7: Write unit test: index a fixture directory with 2 markdown files → assert correct `chunks_new` count.
- [ ] Task 3.8: Write unit test: re-indexing without file changes → `chunks_new = 0`, `chunks_updated = 0`.
- [ ] Task 3.9: Write unit test: re-indexing after editing one section → `chunks_updated = 1`.
- [ ] Task 3.10: Write unit test: deleting a file and re-indexing removes its chunks from `doc_chunks`.

### Phase 4: `changeguard index --docs` Flag
- [ ] Task 4.1: Add `--docs` flag to `IndexArgs` in `src/cli.rs`.
- [ ] Task 4.2: In `execute_index()` in `src/commands/index.rs`, when `--docs` is set, call `run_docs_index()` and print summary.
- [ ] Task 4.3: Print summary: `Docs: {files_crawled} files, {chunks_new} new, {chunks_updated} updated, {chunks_deleted} deleted.`
- [ ] Task 4.4: When `docs.include` is empty, print `No doc paths configured in [docs].include — skipping.` and return `Ok(())`.
- [ ] Task 4.5: Write integration test: run `execute_index` with `--docs` on a fixture repo with a `docs/` directory; assert `chunks_new > 0`.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib docs` — all new tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.
- [ ] Task 5.4: Manually run `changeguard index --docs` on the changeguard repo itself; confirm chunk count is reported.

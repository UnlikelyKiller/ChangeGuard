# Track J9: BM25 Search Snippet Output

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard search <query>` (BM25 full-text search) outputs only file paths and scores:
```
src/commands/scan.rs (score: 4.21)
src/commands/impact.rs (score: 2.87)
```

`changeguard search -r <pattern>` (regex search) outputs file, line number, and matching line content:
```
src/commands/scan.rs:42: fn execute_scan(...) {
```

This inconsistency means BM25 results require a follow-up grep to see why a file matched, doubling the workflow for the most common use case. Engineers using `changeguard search` for code discovery get no context about what matched.

## Fix Strategy
Extend BM25 search to return the top-matching line per result (the line with the highest term overlap) as a snippet. This requires:

1. **`SearchResult` struct**: Add `snippet: Option<String>` and `line_number: Option<usize>` fields.
2. **`tantivy_engine.rs` `search()` method**: After retrieving scored docs, use Tantivy's snippet generation (`SnippetGenerator`) to extract the best matching fragment, then find the corresponding line number by scanning the stored document text.
3. **`src/commands/search.rs`**: Update BM25 output format to `{path}:{line}: {snippet}` when a snippet is available, matching regex output format.

### Snippet generation approach
Tantivy's `SnippetGenerator` can produce HTML-highlighted fragments. For CLI output, strip the highlight tags (`<b>`, `</b>`) and use the plain text. Limit snippets to 120 characters, trimming at word boundaries.

If the content field is not stored in the index (storage disabled), fall back to reading the file from disk and finding the best line via term overlap. This is slightly slower but avoids changing the schema.

Check whether the `content` field in `open_or_create()` has `STORED` set. If not, add `STORED` to the content field and note that the index must be rebuilt after this schema change.

## Scope of Changes

### 1. `src/search/tantivy_engine.rs`
- Check/enable `STORED` on the content field in `open_or_create()`
- In `search()`: for each hit, call `SnippetGenerator::create_for_query(searcher, query, field)?` and `generator.snippet_from_doc(doc)`
- Strip HTML tags from snippet text; truncate to 120 chars at word boundary
- Populate `snippet` and `line_number` in `SearchResult`

### 2. `src/search/mod.rs` (or wherever `SearchResult` is defined)
- Add `snippet: Option<String>` and `line_number: Option<usize>` to `SearchResult`

### 3. `src/commands/search.rs`
- BM25 output: if `result.snippet.is_some()`, print `{path}:{line}: {snippet}`; else fall back to `{path} (score: {score:.2})` for robustness

## Success Criteria
- `changeguard search "temporal coupling"` outputs `{file}:{line}: {matching line}`.
- `changeguard search "storage_cozo"` outputs the line containing the match.
- Snippets are ≤ 120 characters.
- Fallback (no snippet available) shows path + score, not a panic.
- `changeguard search -r "pattern"` output format is unchanged.
- All existing search tests pass.

## Files Changed
- `src/search/tantivy_engine.rs`
- `src/search/mod.rs` (or `SearchResult` definition location)
- `src/commands/search.rs`

## Edge Cases
- **Content field not stored**: Add `STORED` flag; existing index must be rebuilt. Return an `Err` that instructs the user to run `changeguard index --semantic` if the field is missing.
- **Snippet generator returns empty**: Fall back to displaying path + score only (no panic).
- **File on disk changed since indexing**: If line lookup by content fails (line not found in current file), display snippet text only without a line number.
- **Binary or non-UTF-8 file**: `SearchResult.snippet` stays `None`; output falls back to path + score.
- **Very long lines (> 500 chars)**: Truncate snippet at 120 chars with `…` suffix.
- **Multiple strong matches in same file**: Return the single highest-scoring snippet (Tantivy's `SnippetGenerator` handles this by default).

## Definition of Done
- [ ] `changeguard search <query>` outputs `{file}:{line}: {snippet}` for each result.
- [ ] Snippets are ≤ 120 characters.
- [ ] When snippet is unavailable, fallback to `{file} (score: {score:.2})` (no crash).
- [ ] Schema change (STORED content) detected at runtime; user gets a rebuild hint, not a panic.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.

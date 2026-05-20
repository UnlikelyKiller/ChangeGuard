# Track J9 Plan: BM25 Search Snippet Output

## Steps

### Discovery
1. [ ] Read `src/search/tantivy_engine.rs` `open_or_create()` to confirm whether the `content` field has `STORED` set; if not, note that a schema change and index rebuild are required

### Red Phase (failing tests)
2. [ ] Add test in `src/search/tantivy_engine.rs`: index a document with known content, run BM25 search, assert `result.snippet.is_some()` and contains part of the indexed text
3. [ ] Add test: `result.line_number.is_some()` and points to the correct line index
4. [ ] Add test: snippet length ≤ 120 characters
5. [ ] Add test: when content field is not stored / snippet unavailable, `SearchResult.snippet` is `None` and no panic occurs
6. [ ] Run CI gate — new tests expected to fail

### Green Phase — schema
7. [ ] In `open_or_create()`: ensure content field uses `TEXT | STORED` (add `STORED` if not present); add a comment noting index rebuild is required on schema change
8. [ ] On schema mismatch at open time, return `Err` with message directing user to `changeguard index --semantic`

### Green Phase — snippet extraction
9. [ ] Add `snippet: Option<String>` and `line_number: Option<usize>` fields to `SearchResult`
10. [ ] In `search()`: after retrieving scored docs, create `SnippetGenerator::create_for_query(searcher, &query, content_field)?`
11. [ ] For each hit: call `generator.snippet_from_doc(&doc)`; strip `<b>`/`</b>` tags; truncate at 120 chars at word boundary with `…`
12. [ ] Find `line_number`: split stored content by `\n`, find first line containing a query term; store 1-based line number
13. [ ] Populate `SearchResult.snippet` and `SearchResult.line_number`

### Green Phase — output
14. [ ] In `src/commands/search.rs` BM25 output block: `if let Some(snip) = result.snippet { println!("{}:{}: {}", result.path, result.line_number.unwrap_or(0), snip); } else { println!("{} (score: {:.2})", result.path, result.score); }`
15. [ ] Run `cargo build` — fix any type/import errors
16. [ ] Run CI gate — all tests expected to pass

### Verification
17. [ ] `cargo install --path .` to rebuild binary
18. [ ] `changeguard index --semantic` to rebuild index with stored content field
19. [ ] `changeguard search "temporal coupling"` → shows file:line:snippet format
20. [ ] `changeguard search -r "execute_scan"` → regex output unchanged
21. [ ] `changeguard verify` passes

### Finalization
22. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
23. [ ] `changeguard ledger commit` with summary and reason

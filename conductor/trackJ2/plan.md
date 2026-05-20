# Track J2 Plan: Code-Aware Trigram Tokenizer

## Steps

### Red Phase (failing tests)
1. [ ] Add test in `src/search/tantivy_engine.rs` `#[cfg(test)]` block: index a document containing `execute_scan`, call `search_trigrams("execute_scan")`, assert result count ≥ 1 — this test must fail before the fix
2. [ ] Add test: index a document containing `storage_cozo`, assert trigram search finds it
3. [ ] Run CI gate — new tests expected to fail

### Green Phase (implementation)
4. [ ] Add `use tantivy::tokenizer::{WhitespaceTokenizer, LowerCaseFilter, TextAnalyzer};` imports in `src/search/tantivy_engine.rs`
5. [ ] Add `use tantivy::schema::{TextOptions, TextFieldIndexing, IndexRecordOption};` imports
6. [ ] In `open_or_create()`: register `"code_trigram"` tokenizer (`WhitespaceTokenizer + LowerCaseFilter`) on the index immediately after index construction
7. [ ] In `open_or_create()` schema builder: change `trigrams` field from `TEXT` to `TextOptions` using `"code_trigram"` tokenizer with `IndexRecordOption::Basic`
8. [ ] In `open_or_create()`: on `tantivy::TantivyError` caused by unregistered tokenizer (schema mismatch), return a descriptive `Err` recommending `changeguard index --semantic`
9. [ ] Run `cargo build` — fix any import/type errors
10. [ ] Run CI gate — new tests expected to pass, existing tests must still pass

### Verification
11. [ ] `cargo install --path .` to rebuild binary
12. [ ] `changeguard index --semantic` to rebuild index with new tokenizer
13. [ ] `changeguard search -r "execute_scan"` → ≥1 result
14. [ ] `changeguard search -r "fn main"` → still works
15. [ ] `changeguard verify` passes

### Finalization
16. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
17. [ ] `changeguard ledger commit` with summary and reason

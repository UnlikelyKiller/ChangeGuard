# Track I5-1: Fix Regex Search Trigram Case Sensitivity

## Status
In Progress

## Issue
`changeguard search --regex "StorageManager"` returns "No matches found" for valid queries when the pattern contains uppercase characters.

## Root Cause
`TantivySearchEngine::search_trigrams()` in `src/search/tantivy_engine.rs` uses `TermQuery` to search the "trigrams" field. `TermQuery` bypasses Tantivy's default text tokenizer, which lowercases terms during indexing. When trigrams contain uppercase characters (e.g., "Sto" from "StorageManager"), the `TermQuery` searches for the uppercase term, but the index contains lowercase terms ("sto"), causing the BooleanQuery with `Occur::Must` to fail for all uppercase-containing trigrams.

## Fix
Lowercase each trigram before creating the `TermQuery` in `search_trigrams()`.

## Test Plan
1. `cargo test --workspace` — existing tests pass
2. `changeguard search --regex "StorageManager"` — returns matches
3. `changeguard search --regex "struct\s+\w+"` — regex with non-literal components still works

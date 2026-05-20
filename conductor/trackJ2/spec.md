# Track J2: Code-Aware Trigram Tokenizer (Underscore Identifier Fix)

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard search -r "execute_scan"` returns zero results even though the term appears verbatim in the codebase. The same failure affects any pattern containing an underscore-delimited Rust identifier (the dominant naming convention in this codebase).

**Root cause**: Trigrams containing underscores (e.g. `"te_"`, `"e_s"`, `"_sc"` from `execute_scan`) are generated correctly by `src/search/trigram.rs`, stored space-separated in the index, but Tantivy's default `SimpleTokenizer` (which is assigned to the `trigrams` field in `open_or_create()`) treats `_` as a non-alphabetic word-boundary character and splits `"te_"` into `["te"]` — destroying cross-underscore trigrams during ingestion.

The fix is surgical: register a custom tokenizer (`"code_trigram"`) that uses `WhitespaceTokenizer` + `LowerCaser` instead of `SimpleTokenizer`. This preserves every trigram exactly as generated (they are already space-separated), while still normalising case to enable case-insensitive matching.

## Scope of Changes

### 1. Register custom tokenizer
- `src/search/tantivy_engine.rs` → `open_or_create()`: after `let schema = builder.build();` and before index creation, call:
  ```rust
  index.tokenizers().register(
      "code_trigram",
      TextAnalyzer::builder(WhitespaceTokenizer::default())
          .filter(LowerCaseFilter)
          .build(),
  );
  ```

### 2. Assign custom tokenizer to the trigrams field
- `open_or_create()` schema builder: change the `trigrams` field from
  ```rust
  builder.add_text_field("trigrams", TEXT)
  ```
  to
  ```rust
  builder.add_text_field(
      "trigrams",
      TextOptions::default()
          .set_indexing_options(
              TextFieldIndexing::default()
                  .set_tokenizer("code_trigram")
                  .set_index_option(IndexRecordOption::Basic),
          ),
  )
  ```
- Field need not be stored (trigrams are for pre-filtering only).

### 3. Rebuild index after registration
- The registration must happen before any write or search operations on the index.
- If the index already exists on disk with the old tokenizer schema, existing docs are stale but the index is not corrupt; a rebuild via `changeguard index --semantic` is sufficient.
- Add a migration note in the `open_or_create()` doc comment.

### 4. Query-side: ensure term lowercasing
- `search_trigrams()` already lowercases before creating `TermQuery` (Track I5-1). No change needed.

## Success Criteria
- `changeguard search -r "execute_scan"` returns matches when the term exists in the codebase.
- `changeguard search -r "storage_cozo"` returns matches.
- `changeguard search -r "fn main"` (no underscore) continues to work.
- `changeguard search -r "[A-Z][a-z]+"` (regex without underscores) continues to work.
- All existing search tests pass.

## Files Changed
- `src/search/tantivy_engine.rs`

## Edge Cases
- **Existing index on disk**: The tokenizer schema change is not backward compatible at the segment level. A user who upgrades without rebuilding will see the same zero-result bug. The `open_or_create()` path should detect schema mismatch (Tantivy raises an error when the tokenizer referenced by a field is unregistered) and return a descriptive `Err` that instructs the user to run `changeguard index --semantic` to rebuild.
- **Empty trigrams field**: If a document has no trigrams (e.g. a binary file that was indexed), `add_text(trigrams_field, "")` is a no-op for Tantivy; the fix does not affect this path.
- **Case sensitivity**: `WhitespaceTokenizer + LowerCaseFilter` ensures `"te_"` and `"TE_"` map to the same token. Query side already lowercases, so this is consistent.
- **Index rebuild after first install**: On first run the index is created fresh and the tokenizer is registered before any document is added; no migration issue.

## Definition of Done
- [ ] `changeguard search -r "execute_scan"` returns ≥1 result (after `changeguard index --semantic` rebuilds the index).
- [ ] `changeguard search -r "storage_cozo"` returns ≥1 result.
- [ ] Schema mismatch (pre-fix index + post-fix binary) produces a clear error message pointing to `changeguard index --semantic`.
- [ ] No regression: all existing passing search tests still pass.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.

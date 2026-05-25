# Track CR1 Plan: Incremental Semantic Indexing Deletions

## Phase 1: Implementation
- [x] Retrieve list of all tracked files from `semantic_file_hash` table before starting incremental walk in `src/commands/index.rs`.
- [x] Compare this list with the active files found in the repository.
- [x] For each file that exists in the database but is missing from the workspace:
  - [x] Invoke `semantic.remove_file_snippets(file_path)` to clean up vector embeddings.
  - [x] Delete the row from the `semantic_file_hash` relation in CozoDB.

## Phase 2: Testing & Verification
- [x] Create a regression test in `tests/semantic_search.rs` that:
  - [x] Indexes two files.
  - [x] Removes one file from the filesystem.
  - [x] Runs `index --semantic --incremental`.
  - [x] Verifies the deleted file's snippets are completely pruned and cannot be retrieved via query.
- [x] Run `cargo test` to verify — all tests pass.

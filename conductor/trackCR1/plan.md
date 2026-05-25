# Track CR1 Plan: Incremental Semantic Indexing Deletions

## Phase 1: Implementation
- [ ] Retrieve list of all tracked files from `semantic_file_hash` table before starting incremental walk in `src/commands/index.rs`.
- [ ] Compare this list with the active files found in the repository.
- [ ] For each file that exists in the database but is missing from the workspace:
  - [ ] Invoke `semantic.remove_file_snippets(file_path)` to clean up vector embeddings.
  - [ ] Delete the row from the `semantic_file_hash` relation in CozoDB.

## Phase 2: Testing & Verification
- [ ] Create a regression test in `tests/semantic_search.rs` that:
  - [ ] Indexes two files.
  - [ ] Removes one file from the filesystem.
  - [ ] Runs `index --semantic --incremental`.
  - [ ] Verifies the deleted file's snippets are completely pruned and cannot be retrieved via query.
- [ ] Run `cargo test` to verify.

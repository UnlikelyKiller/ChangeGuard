# Track CR1: Incremental Semantic Indexing Deletions

## Status
Planned

## Milestone
CR: Codex Review Remediation

## Problem
During incremental semantic indexing (`index --semantic --incremental`), ChangeGuard only processes files currently present on the filesystem. If a file is deleted, its old vector embeddings and content hashes are never pruned, leading to stale search results.

## Objective
Update the incremental semantic indexing flow to identify deleted files, prune their stale vector embeddings from the CozoDB database, and remove their tracking hashes from the `semantic_file_hash` relation.

## Scope
- Modify `src/commands/index.rs` to detect files that were previously indexed but are now missing.
- Call `remove_file_snippets` and delete the corresponding rows in the `semantic_file_hash` table for those deleted files.
- Add regression tests to verify that deleted files are cleanly pruned during incremental runs.

## Success Criteria
- [ ] Deleting an indexed file and running `changeguard index --semantic --incremental` successfully removes its embeddings.
- [ ] Deleted files do not reappear in subsequent semantic search queries.
- [ ] An integration test is added that asserts complete deletion of snippets on file removal during incremental sync.

## Definition of Done
- [ ] Deletion detection added to incremental indexing in `src/commands/index.rs`.
- [ ] Clean-up logic implemented for `semantic_file_hash` and `remove_file_snippets` inside the deletion check.
- [ ] Regression test added to `tests/semantic_search.rs`.
- [ ] `cargo test` passes.

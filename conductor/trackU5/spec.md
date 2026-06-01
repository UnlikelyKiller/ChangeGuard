# Track U5 Spec: Interactive Index Recovery

## Background
When a user runs `search --semantic` or `ask` in a new repository (or after a reset), they often encounter a warning that the "Semantic index is empty." The current flow requires them to manually run `changeguard index --semantic`, which is a friction point.

## Objective
Implement an interactive prompt (or an automatic trigger) that offers to run the indexing process when a semantic operation is attempted on an empty or stale index.

## Proposed Design
* In `src/commands/search.rs` and `src/commands/ask.rs`, check the index status using `VectorStore::is_empty()`.
* Use the `inquire` crate (Confirm prompt) to ask if indexing should start immediately.
* If the user confirms, invoke `execute_index(true, false)` (semantic index) before proceeding with the search.
* Support an `--auto-index` flag to bypass the prompt and always index if needed.

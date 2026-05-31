## Plan: Search UX & Discoverability Overhaul
### Phase 1: Explicit First-Use Auto-Indexing
- [x] Task 1.1: In `src/commands/search.rs`, locate the auto-indexing block (`if args.index || engine.document_count() == 0 { ... }`).
- [x] Task 1.2: Add a user-facing log or utilize `src::ui::spinner::Spinner` to inform the user that the index is being built for the first time.
- [x] Task 1.3: Emit a success message (e.g., "Index built successfully.") once the stream indexing completes and before the search continues.

### Phase 2: Empty Search Results Guidance
- [x] Task 2.1: In `perform_search` within `src/commands/search.rs`, update the `if matches.is_empty()` branch for regex searches. Provide guidance on checking regex syntax and suggesting an index update if changes are missing.
- [x] Task 2.2: In the same function, update the `if results.is_empty()` branch for BM25 searches. Suggest using the `--regex` (`-r`) flag for partial/literal matching, and suggest an index update.
- [x] Task 2.3: Ensure the new guidance messages use `owo_colors` for consistency (e.g., yellow for hints, cyan for command highlights).
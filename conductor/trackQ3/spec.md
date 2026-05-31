# Track Q3: Search UX & Discoverability Overhaul

## Objective
Enhance the user experience of the `changeguard search` command by providing actionable guidance when no results are found, and by making the implicit first-time indexing visible and explicit to the user.

## Requirements

1.  **Empty Results Guidance:**
    *   When a BM25 (default) search yields zero results, print a helpful message instead of just "No matches found."
    *   Guidance should suggest using the `--regex` (`-r`) flag for partial/literal substring matches.
    *   Guidance should suggest running `changeguard index` if the user suspects recent changes are missing.
    *   When a Regex search yields zero results, the guidance should advise checking the regex syntax or running an index update.
2.  **Explicit First-Use Auto-Indexing:**
    *   The `search` command currently performs silent inline indexing if the Tantivy index is empty (`engine.document_count() == 0`). This appears as a hang to the user because only `debug!` logs are emitted.
    *   Add a user-facing terminal message (e.g., using `owo_colors` or `src::ui::spinner::Spinner`) before this indexing occurs to indicate "Building index for first use...".
    *   Communicate completion once the auto-indexing finishes so the user knows the search query is about to execute.

## Testing Strategy
1.  **Empty Regex Search**: Run `changeguard search -r "NONEXISTENT_ASDF"` and verify the new guidance is printed.
2.  **Empty BM25 Search**: Run `changeguard search "NONEXISTENT_ASDF"` and verify the suggestion to try `--regex` is present.
3.  **First-use Indexing**: Delete the search index (e.g. `.changeguard/search_index`) and execute `changeguard search foo`. Verify the explicit "Building index" message is displayed instead of a silent delay.
# Track CG-F5: Search UX & Fallback Enhancements

## Phase 1: Fuzzy Search Fallback
- [x] Implement fuzzy search matching logic within the Tantivy engine or as a post-query filter.
- [x] Update `execute_search` to detect 0 BM25 results and automatically trigger the fuzzy search.
- [x] Add CLI output to indicate "Falling back to fuzzy search...".
- [x] Write integration tests for fuzzy search fallback.

## Phase 2: Semantic Search Handoff Hint
- [x] In `execute_search`, if 0 results remain after fuzzy fallback, output the hint: `HINT: No exact symbols found. Try semantic search instead: changeguard ask "<query>"`.
- [x] Verify the hint is rendered correctly in the terminal output without requesting user input.

## Phase 3: Proactive Scan Output
- [x] Locate the clean tree success message in `execute_scan`.
- [x] Add proactive guidance text: "HINT: Run `changeguard ledger status` to check for pending transactions, or `changeguard index` to update your local graph."
- [x] Verify the updated output format in `scan` tests.

## Phase 4: Finalization
- [x] Run `changeguard verify`.
- [x] Update conductor status to Completed.

# Track CG-F5: Search UX & Fallback Enhancements

## Objective
Improve the discoverability and user experience of `changeguard search` and `changeguard scan` by implementing fuzzy fallback for strict BM25 searches, interactive semantic search handoffs, and proactive next-step guidance for clean scans.

## Requirements

1. **Fuzzy Search Fallback**:
   - When `changeguard search` is executed without `--regex` and yields 0 results using strict BM25, the engine must automatically retry the query using a fuzzy matching algorithm (or substring match) before returning an empty result.
   - The output should clearly indicate when it has fallen back to fuzzy search.

2. **Semantic Search Handoff Hint**:
   - If both strict and fuzzy searches yield 0 results, the CLI should output a helpful hint rather than using an interactive prompt (to prevent hanging headless AI agents).
   - The output should display: `HINT: No exact symbols found. Try semantic search instead: changeguard ask "<query>"`

3. **Proactive Scan Output Guidance**:
   - When `changeguard scan --impact` runs on a completely clean working tree, the success output currently tells the user to `git add <files>`.
   - Update this success message to proactively suggest running `changeguard ledger status` or `changeguard index` to ensure the user is aware of pending ledger items or stale indices.

## Definition of Done
- `changeguard search <query>` automatically retries with fuzzy matching when strict BM25 fails.
- Interactive terminal sessions prompt for semantic search fallback when no results are found.
- Clean `changeguard scan --impact` output includes suggestions for `ledger status` and `index`.
- Unit and integration tests cover the new fallback logic and output.
- All existing CI gates and verification pass.

## Plan: Fix ask Gemini Model Name (Track CG-F1)

### Summary
`changeguard ask "..."` fails with HTTP 404: "models/gemini-1.5-flash is not found for API version v1beta". The hardcoded model name is deprecated. Additionally, the AI-Brains bridge query within `ask` fails with "fts5: syntax error near '?'", preventing the full context-gathering pipeline from running.

### Phase 1: Fix Gemini Model
- [ ] Task 1.1: Find where `gemini-1.5-flash` is hardcoded or defaulted (likely `src/commands/ask.rs` or a Gemini client module)
- [ ] Task 1.2: Replace with `gemini-2.5-flash` (current supported model as of 2026-05)
- [ ] Task 1.3: Make model name configurable via `.changeguard/config.toml` `[ask]` section or `CHANGEGUARD_GEMINI_MODEL` env var
- [ ] Task 1.4: Test: `changeguard ask "test" --backend gemini` returns a valid response (requires API key; skip if unavailable, verify no 404)

### Phase 2: Fix AI-Brains Bridge Query in ask Context
- [ ] Task 2.1: Trace the FTS5 syntax error ("fts5: syntax error near '?'") in the ask context-gathering pipeline
- [ ] Task 2.2: Ensure query sanitization escapes special characters before passing to AI-Brains FTS5
- [ ] Task 2.3: The bridge query in `ask` context uses `changeguard bridge query <QUERY>` which calls AI-Brains — verify this path works or falls back gracefully

### Phase 3: Gate
- [ ] Task 3.1: `cargo fmt --all -- --check` passes
- [ ] Task 3.2: `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Task 3.3: `cargo test --workspace` passes

## Plan: Track M3-2 — Ask Backend Routing & Integration

### Phase 1: Backend Enum & Config
- [ ] Task 1.1: Add `Backend` enum to `src/commands/ask.rs` (or `src/gemini/modes.rs`): `Local`, `Gemini`.
- [ ] Task 1.2: Add `--backend` flag to `AskArgs` in `src/cli.rs` accepting `local` or `gemini` (case-insensitive).
- [ ] Task 1.3: Write unit test: `--backend local` parses to `Backend::Local`.
- [ ] Task 1.4: Write unit test: `--backend gemini` parses to `Backend::Gemini`.

### Phase 2: Auto-Selection Logic
- [ ] Task 2.1: Implement `resolve_backend(config: &Config, explicit: Option<Backend>) -> Backend` in `src/commands/ask.rs`.
- [ ] Task 2.2: If `explicit` is `Some`, return it.
- [ ] Task 2.3: If `config.local_model.prefer_local = true` and `base_url` non-empty: return `Local`.
- [ ] Task 2.4: If no Gemini API key found (no env var, no `.env`, no config key) and `base_url` non-empty: return `Local`.
- [ ] Task 2.5: Otherwise: return `Gemini`.
- [ ] Task 2.6: Write unit test: `prefer_local = true` → `Local`.
- [ ] Task 2.7: Write unit test: no API key, `base_url` set → `Local`.
- [ ] Task 2.8: Write unit test: API key present, no explicit flag → `Gemini`.
- [ ] Task 2.9: Write unit test: `--backend gemini` with no API key → `Gemini` (explicit overrides auto).

### Phase 3: Local Backend Execution Path
- [ ] Task 3.1: In `execute_ask()`, after resolving the backend, branch: if `Backend::Local`, call `assemble_context()` + `local_model::client::complete()`.
- [ ] Task 3.2: Print `\n{Local Model Response:}` (bold green, matching Gemini output header style).
- [ ] Task 3.3: On local model `Err`: print the error message and return `Err` (same pattern as Gemini failure).
- [ ] Task 3.4: All four modes (`Analyze`, `Suggest`, `ReviewPatch`, `Narrative`) must work with `Backend::Local` — verify each routes to `assemble_context` correctly.
- [ ] Task 3.5: Write integration test: mock llama-server at `localhost` returns canned response → `execute_ask` with `Backend::Local` prints it and returns `Ok`.
- [ ] Task 3.6: Write test: `Backend::Local` with server unreachable → returns `Err` with clear message.
- [ ] Task 3.7: Write test: `ReviewPatch` mode with `Backend::Local` on a clean working tree → falls back to general analysis (no diff), not an error.

### Phase 4: `changeguard config verify` Extension
- [ ] Task 4.1: Extend the `config verify` subcommand output to include:
  ```
  Ask backend:   Gemini (API key present)
  ```
  or:
  ```
  Ask backend:   Local (http://localhost:8080, prefer_local=true)
  ```
- [ ] Task 4.2: Write unit test: `resolve_backend` output is correctly described in verify output.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib` — all new tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.
- [ ] Task 5.4: Manually test `changeguard ask --backend local "what changed?"` with llama-server running; confirm response is printed cleanly without CLI noise.

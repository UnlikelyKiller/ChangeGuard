# Upgrade Notes

These notes summarize dependency-specific cautions for ChangeGuard maintenance.

## `rusqlite`

- `0.39.x` tightened statement validation.
- Keep SQL statements single-purpose and avoid multi-statement strings.
- Store integer durations and booleans as signed SQLite-compatible values.
- Read-only daemon access must not attempt write-capable PRAGMAs.

## `thiserror`

- `2.x` removed some raw-identifier formatting behavior.
- Prefer normal field names in formatted error messages.

## `gix`

- `gix` remains high-churn pre-1.0.
- Verify status/diff API assumptions against the pinned version before refactoring git classification.

## `tree-sitter`

- Update parsers as a coordinated family.
- Re-run parser fixtures across Rust, TypeScript, and Python on every bump.
- Re-run complexity edge tests for syntax errors, unsupported files, and large-file caps.

## `tower-lsp-server`

- The daemon uses `tower-lsp-server` 0.23 behind the optional `daemon` feature.
- Keep Tokio feature requirements in sync with daemon lifecycle tasks.
- Re-run `cargo test --all-features --test daemon_lifecycle -- --test-threads=1` after LSP changes.

## Gemini CLI

- `changeguard ask` shells out to `gemini analyze`.
- Missing CLI errors must remain actionable: `Gemini CLI not found. Install Gemini CLI to enable narrative summaries.`
- Narrative mode should use a single structured prompt, not the generic question template.

## General

- Commit `Cargo.lock`.
- Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features -j 1 -- --test-threads=1` after dependency changes.
- Treat watcher and subprocess behavior as platform-sensitive and verify on Windows.

# Upgrade Notes

These notes summarize dependency-specific cautions for ChangeGuard maintenance.

## `rusqlite`

- `0.39.x` tightened statement validation.
- Keep SQL statements single-purpose and avoid multi-statement strings.

## `thiserror`

- `2.x` removed some raw-identifier formatting behavior.
- Prefer normal field names in formatted error messages.

## `gix`

- `gix` remains high-churn pre-1.0.
- Verify status/diff API assumptions against the pinned version before refactoring git classification.

## `tree-sitter`

- Update parsers as a coordinated family.
- Re-run parser fixtures across Rust, TypeScript, and Python on every bump.

## General

- Commit `Cargo.lock`.
- Run `cargo clippy --all-targets --all-features` and `cargo test -j 1 -- --test-threads=1` after dependency changes.
- Treat watcher and subprocess behavior as platform-sensitive and verify on Windows.

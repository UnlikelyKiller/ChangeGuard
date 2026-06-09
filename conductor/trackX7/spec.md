# Track X7: `doctor` Shows Placeholder When Embedding Model Is Unconfigured

**Status:** Completed  
**Milestone:** X — Command Surface Correctness  
**Priority:** Low

## Objective

`changeguard doctor` shows `"  (768 dims) @ http://localhost:11434"` with no model name when `local_model.embedding_model` is not set in `config.toml`. The blank prefix makes the status line look malformed. A clear `(not configured)` placeholder is needed.

## Problem Statement

`LocalModelConfig::default()` sets `embedding_model: String::new()`. When the user has not set this in `config.toml`, `resolve_string(&config.embedding_model, "CHANGEGUARD_EMBEDDING_MODEL")` also returns `""`. The doctor format string:
```rust
format!("{} ({} dims) @ {}", config.local_model.embedding_model, dims.dimensions, url)
```
produces `" (768 dims) @ http://..."` — blank name, leading space.

## Acceptance Criteria

1. When `embedding_model` resolves to `""`, `doctor` prints `"(not configured) (768 dims) @ http://..."` instead of the blank-prefixed form.
2. When `generation_model` resolves to `""`, `doctor` prints `"(not configured) @ http://..."`.
3. The `(not configured)` text is styled in yellow to distinguish it from a successful model name.
4. No change to the data model — this is purely a display fix.

## Key Files

- `src/commands/doctor.rs` — lines 44–56 (embedding probe), lines 58–68 (completion probe)

## Definition of Done

- `changeguard doctor` shows `(not configured)` in yellow when model name is blank, even if the endpoint is reachable.
- `cargo nextest run --lib --bins --workspace` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

# Track X7 Plan: `doctor` Embedding Model Name Placeholder

## Phase 1 — Implementation (no failing test needed — display only)
- [x] 1. In `src/commands/doctor.rs`, extract a helper at the top of `execute_doctor`:
  ```rust
  fn model_name_or_placeholder(name: &str) -> String {
      if name.is_empty() {
          "(not configured)".yellow().to_string()
      } else {
          name.to_string()
      }
  }
  ```
- [x] 2. In the embedding probe `Ok(dims)` arm (line ~45), replace:
  ```rust
  config.local_model.embedding_model,
  ```
  with:
  ```rust
  model_name_or_placeholder(&config.local_model.embedding_model),
  ```
- [x] 3. Apply the same to `generation_model` in the completion probe display.

## Phase 2 — Verification
- [x] 4. Temporarily remove `embedding_model` from `config.toml`, run `changeguard doctor`, confirm `(not configured)` appears in yellow.
- [x] 5. Restore config, run `changeguard doctor`, confirm normal model name displays.
- [x] 6. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 7. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 8. Run `cargo fmt --all -- --check` — clean.
- [x] 9. Update `conductor/conductor.md` status to Completed.
